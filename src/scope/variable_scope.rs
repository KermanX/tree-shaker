use super::{cf_scope::CfScope, exhaustive::TrackerRunner};
use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  consumable::{box_consumable, Consumable, ConsumableTrait},
  entity::{Entity, ForwardedEntity, LiteralEntity, UnionEntity, UnknownEntity},
};
use oxc::semantic::{ScopeId, SymbolId};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{fmt, mem};

#[derive(Debug)]
pub struct Variable<'a> {
  pub kind: DeclarationKind,
  pub exhausted: bool,
  pub value: Option<Entity<'a>>,
  pub decl_dep: Consumable<'a>,
}

/// It's not good to clone, but it's fine for now
impl Clone for Variable<'_> {
  fn clone(&self) -> Self {
    Self {
      kind: self.kind,
      exhausted: self.exhausted,
      value: self.value.clone(),
      decl_dep: self.decl_dep.cloned(),
    }
  }
}

pub struct VariableScope<'a> {
  /// Cf scopes when the scope was created
  pub cf_scope: ScopeId,
  pub variables: FxHashMap<SymbolId, Variable<'a>>,
  pub exhaustive_deps: FxHashMap<SymbolId, FxHashSet<TrackerRunner<'a>>>,
}

impl fmt::Debug for VariableScope<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut map = f.debug_map();
    for (k, v) in self.variables.iter() {
      map.entry(&k, &format!("{:?} {}", v.kind, v.value.is_some()));
    }
    map.finish()
  }
}

impl<'a> VariableScope<'a> {
  pub fn new(cf_scope: ScopeId) -> Self {
    Self { cf_scope, variables: Default::default(), exhaustive_deps: Default::default() }
  }
}

impl<'a> Analyzer<'a> {
  fn declare_on_scope(
    &mut self,
    id: ScopeId,
    kind: DeclarationKind,
    symbol: SymbolId,
    decl_dep: Consumable<'a>,
    fn_value: Option<Entity<'a>>,
  ) {
    if let Some(old) = self.scope_context.variable.get(id).variables.get(&symbol) {
      // Here we can't use kind.is_untracked() because this time we are declaring a variable
      if old.kind.is_untracked() {
        self.consume(decl_dep);
        fn_value.map(|val| val.consume(self));
        return;
      }

      if old.kind.is_shadowable() && kind.is_redeclarable() {
        // Redeclaration is sometimes allowed
        // var x = 1; var x = 2;
        // function f(x) { var x }
        let variable = self.scope_context.variable.get_mut(id).variables.get_mut(&symbol).unwrap();
        variable.kind = kind;
        // TODO: Sometimes this is not necessary
        variable.decl_dep = box_consumable((variable.decl_dep.cloned(), decl_dep));
        if let Some(new_val) = fn_value {
          self.write_on_scope((self.scope_context.variable.current_depth(), id), symbol, &new_val);
        }
      } else {
        let decl_dep = (old.decl_dep.cloned(), decl_dep);
        let name = self.semantic.symbols().get_name(symbol);
        self.thrown_builtin_error(format!("Variable {name:?} already declared"));
        self.consume(decl_dep);
      }
    } else {
      let has_fn_value = fn_value.is_some();
      self
        .scope_context
        .variable
        .get_mut(id)
        .variables
        .insert(symbol, Variable { kind, exhausted: false, value: fn_value, decl_dep });
      if has_fn_value {
        self.exec_exhaustive_deps(false, (id, symbol));
      }
    }
  }

  fn init_on_scope(
    &mut self,
    id: ScopeId,
    symbol: SymbolId,
    value: Option<Entity<'a>>,
    init_dep: Consumable<'a>,
  ) {
    let variable = self.scope_context.variable.get_mut(id).variables.get_mut(&symbol).unwrap();

    if variable.exhausted {
      if let Some(value) = value {
        self.consume(value);
      }
      self.consume(init_dep);
    } else if variable.kind.is_redeclarable() {
      if let Some(value) = value {
        self.write_on_scope(
          (self.scope_context.variable.current_depth(), id),
          symbol,
          &ForwardedEntity::new(value, init_dep),
        );
      } else {
        // Do nothing
      }
    } else {
      variable.value =
        Some(ForwardedEntity::new(value.unwrap_or_else(LiteralEntity::new_undefined), init_dep));
      self.exec_exhaustive_deps(false, (id, symbol));
    }
  }

  /// None: not in this scope
  /// Some(None): in this scope, but TDZ
  /// Some(Some(val)): in this scope, and val is the value
  fn read_on_scope(&mut self, id: ScopeId, symbol: SymbolId) -> Option<Option<Entity<'a>>> {
    self.scope_context.variable.get(id).variables.get(&symbol).cloned().map(|variable| {
      let value = variable.value.clone().or_else(|| {
        variable
          .kind
          .is_var()
          .then(|| ForwardedEntity::new(LiteralEntity::new_undefined(), variable.decl_dep.cloned()))
      });

      let target_cf_scope =
        self.find_first_different_cf_scope(self.scope_context.variable.get(id).cf_scope);
      self.mark_exhaustive_read((id, symbol), target_cf_scope);

      if value.is_none() {
        // TDZ
        self.consume(variable.decl_dep.cloned());
        self.handle_tdz(target_cf_scope);
      }

      value
    })
  }

  fn write_on_scope(
    &mut self,
    (depth, id): (usize, ScopeId),
    symbol: SymbolId,
    new_val: &Entity<'a>,
  ) -> bool {
    if let Some(variable) = self.scope_context.variable.get(id).variables.get(&symbol).cloned() {
      let new_val = new_val.clone();
      if variable.kind.is_untracked() {
        self.consume(new_val);
      } else if variable.kind.is_const() {
        self.thrown_builtin_error("Cannot assign to const variable");
        self.consume(variable.decl_dep);
        new_val.consume(self);
      } else {
        let target_cf_scope =
          self.find_first_different_cf_scope(self.scope_context.variable.get(id).cf_scope);
        let dep = (self.get_assignment_dep(depth), variable.decl_dep.cloned());

        if variable.exhausted {
          self.consume(dep);
          self.consume(new_val);
        } else {
          let old_val = variable.value;
          let should_consume = if old_val.is_some() {
            // Normal write
            self.mark_exhaustive_write((id, symbol), target_cf_scope)
          } else if !variable.kind.is_redeclarable() {
            // TDZ write
            self.handle_tdz(target_cf_scope);
            true
          } else {
            // Write uninitialized `var`
            self.mark_exhaustive_write((id, symbol), target_cf_scope)
          };

          if should_consume {
            self.consume(dep);
            self.consume(new_val);
            if let Some(old_val) = &old_val {
              old_val.consume(self);
            }

            let variable =
              self.scope_context.variable.get_mut(id).variables.get_mut(&symbol).unwrap();
            variable.exhausted = true;
            variable.value = Some(UnknownEntity::new_unknown());
          } else {
            let indeterminate = self.is_relatively_indeterminate(target_cf_scope);

            let variable =
              self.scope_context.variable.get_mut(id).variables.get_mut(&symbol).unwrap();
            variable.value = Some(ForwardedEntity::new(
              if indeterminate {
                UnionEntity::new(vec![
                  old_val.unwrap_or_else(LiteralEntity::new_undefined),
                  new_val,
                ])
              } else {
                new_val
              },
              box_consumable(dep),
            ));
          };

          self.exec_exhaustive_deps(should_consume, (id, symbol));
        }
      }
      true
    } else {
      false
    }
  }

  pub fn consume_on_scope(&mut self, id: ScopeId, symbol: SymbolId) -> bool {
    if let Some(variable) = self.scope_context.variable.get(id).variables.get(&symbol).cloned() {
      if !variable.exhausted {
        variable.decl_dep.consume(self);
        if let Some(value) = &variable.value {
          value.consume(self);
        }
        let variable = self.scope_context.variable.get_mut(id).variables.get_mut(&symbol).unwrap();
        variable.exhausted = true;
        variable.value = Some(UnknownEntity::new_unknown());
      }
      true
    } else {
      false
    }
  }

  fn mark_untracked_on_scope(&mut self, id: ScopeId, symbol: SymbolId) {
    let old = self.scope_context.variable.get_mut(id).variables.insert(
      symbol,
      Variable {
        exhausted: true,
        kind: DeclarationKind::UntrackedVar,
        value: Some(UnknownEntity::new_unknown()),
        decl_dep: box_consumable(()),
      },
    );
    debug_assert!(old.is_none());
  }
}

impl<'a> Analyzer<'a> {
  pub fn declare_symbol(
    &mut self,
    symbol: SymbolId,
    decl_dep: Consumable<'a>,
    exporting: bool,
    kind: DeclarationKind,
    fn_value: Option<Entity<'a>>,
  ) {
    if exporting {
      self.named_exports.push(symbol);
    }
    if kind == DeclarationKind::FunctionParameter {
      self.call_scope_mut().args.1.push(symbol);
    }
    if kind == DeclarationKind::Var {
      self.insert_var_decl(symbol);
    }

    let variable_scope = self.get_variable_scope(kind.is_var());
    self.declare_on_scope(variable_scope, kind, symbol, decl_dep, fn_value);
  }

  pub fn init_symbol(
    &mut self,
    symbol: SymbolId,
    value: Option<Entity<'a>>,
    init_dep: Consumable<'a>,
  ) {
    let flags = self.semantic.symbols().get_flags(symbol);
    let is_function_scope = flags.is_function_scoped_declaration() && !flags.is_catch_variable();
    let variable_scope = self.get_variable_scope(is_function_scope);
    self.init_on_scope(variable_scope, symbol, value, init_dep);
  }

  fn get_variable_scope(&self, is_function_scope: bool) -> ScopeId {
    if is_function_scope {
      self.call_scope().body_variable_scope
    } else {
      self.scope_context.variable.current_id()
    }
  }

  /// `None` for TDZ
  pub fn read_symbol(&mut self, symbol: SymbolId) -> Option<Entity<'a>> {
    for id in self.scope_context.variable.stack.clone().into_iter().rev() {
      if let Some(value) = self.read_on_scope(id, symbol) {
        return value;
      }
    }
    self.mark_unresolved_reference(symbol);
    Some(UnknownEntity::new_unknown())
  }

  pub fn write_symbol(&mut self, symbol: SymbolId, new_val: Entity<'a>) {
    for id in self.scope_context.variable.stack.clone().into_iter().enumerate().rev() {
      if self.write_on_scope(id, symbol, &new_val) {
        return;
      }
    }
    self.consume(new_val);
    self.mark_unresolved_reference(symbol);
  }

  fn mark_unresolved_reference(&mut self, symbol: SymbolId) {
    if self.semantic.symbols().get_flags(symbol).is_function_scoped_declaration() {
      self.insert_var_decl(symbol);
      let id = self.get_variable_scope(true);
      self.mark_untracked_on_scope(id, symbol);
    } else {
      self.thrown_builtin_error("Unresolved identifier reference");
    }
  }

  fn insert_var_decl(&mut self, symbol: SymbolId) {
    let key = self.call_scope().source;
    self.var_decls.entry(key).or_default().insert(symbol);
  }

  pub fn handle_tdz(&mut self, target_cf_scope: usize) {
    if self.has_exhaustive_scope_since(target_cf_scope) {
      self.may_throw();
    } else {
      self.thrown_builtin_error("Cannot access variable before initialization");
    }
    self.refer_global();
  }

  pub fn refer_global(&mut self) {
    self.may_throw();
    for id in self.scope_context.cf.stack.clone() {
      let mut deps = mem::take(&mut self.scope_context.cf.get_mut(id).deps);
      deps.consume_all(self);
    }
  }

  pub fn refer_to_diff_variable_scope(&mut self, another: ScopeId) {
    let target_depth = self.find_first_different_variable_scope(another);
    let dep = self.get_assignment_dep(target_depth);
    self.consume(dep);
  }
}
