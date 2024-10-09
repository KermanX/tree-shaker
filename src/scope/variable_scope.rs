use super::exhaustive::TrackerRunner;
use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  entity::{Consumable, Entity, UNDEFINED_ENTITY},
};
use oxc::semantic::{ScopeId, SymbolId};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{fmt, mem};

/// It's not good to clone, but it's fine for now
#[derive(Debug, Clone)]
pub struct Variable<'a> {
  pub kind: DeclarationKind,
  pub exhausted: bool,
  pub value: Option<Entity<'a>>,
  pub decl_dep: Consumable<'a>,
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
        variable.decl_dep = (variable.decl_dep.clone(), decl_dep).into();
        if let Some(new_val) = fn_value {
          self.write_on_scope((self.scope_context.variable.current_depth(), id), symbol, new_val);
        }
      } else {
        let decl_dep = (old.decl_dep.clone(), decl_dep);
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
          self.factory.new_computed(value, init_dep),
        );
      } else {
        // Do nothing
      }
    } else {
      variable.value =
        Some(self.factory.new_computed(value.unwrap_or(self.factory.undefined), init_dep));
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
          .then(|| self.factory.new_computed(self.factory.undefined, variable.decl_dep.clone()))
      });

      let target_cf_scope =
        self.find_first_different_cf_scope(self.scope_context.variable.get(id).cf_scope);
      self.mark_exhaustive_read((id, symbol), target_cf_scope);

      if value.is_none() {
        // TDZ
        self.consume(variable.decl_dep.clone());
        self.handle_tdz(target_cf_scope);
      }

      value
    })
  }

  fn write_on_scope(
    &mut self,
    (depth, id): (usize, ScopeId),
    symbol: SymbolId,
    new_val: Entity<'a>,
  ) -> bool {
    if let Some(variable) = self.scope_context.variable.get(id).variables.get(&symbol).cloned() {
      if variable.kind.is_untracked() {
        self.consume(new_val);
      } else if variable.kind.is_const() {
        self.thrown_builtin_error("Cannot assign to const variable");
        self.consume(variable.decl_dep);
        new_val.consume(self);
      } else {
        let target_cf_scope =
          self.find_first_different_cf_scope(self.scope_context.variable.get(id).cf_scope);
        let dep = self.get_assignment_deps(depth, variable.decl_dep.clone());

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
            variable.value = Some(self.factory.unknown);
          } else {
            let indeterminate = self.is_relatively_indeterminate(target_cf_scope);

            let variable =
              self.scope_context.variable.get_mut(id).variables.get_mut(&symbol).unwrap();
            variable.value = Some(self.factory.new_computed(
              if indeterminate {
                self.factory.new_union(vec![
                  old_val.unwrap_or(unsafe { mem::transmute(UNDEFINED_ENTITY) }),
                  new_val,
                ])
              } else {
                new_val
              },
              dep,
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
        variable.value = Some(self.factory.unknown);
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
        value: Some(self.factory.unknown),
        decl_dep: ().into(),
      },
    );
    debug_assert!(old.is_none());
  }
}

impl<'a> Analyzer<'a> {
  pub fn declare_symbol(
    &mut self,
    symbol: SymbolId,
    decl_dep: impl Into<Consumable<'a>>,
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
    self.declare_on_scope(variable_scope, kind, symbol, decl_dep.into(), fn_value);
  }

  pub fn init_symbol(
    &mut self,
    symbol: SymbolId,
    value: Option<Entity<'a>>,
    init_dep: impl Into<Consumable<'a>>,
  ) {
    let flags = self.semantic.symbols().get_flags(symbol);
    let is_function_scope = flags.is_function_scoped_declaration() && !flags.is_catch_variable();
    let variable_scope = self.get_variable_scope(is_function_scope);
    self.init_on_scope(variable_scope, symbol, value, init_dep.into());
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
    for depth in (0..self.scope_context.variable.stack.len()).rev() {
      let id = self.scope_context.variable.stack[depth];
      if let Some(value) = self.read_on_scope(id, symbol) {
        return value;
      }
    }
    self.mark_unresolved_reference(symbol);
    Some(self.factory.unknown)
  }

  pub fn write_symbol(&mut self, symbol: SymbolId, new_val: Entity<'a>) {
    for depth in (0..self.scope_context.variable.stack.len()).rev() {
      let id = self.scope_context.variable.stack[depth];
      if self.write_on_scope((depth, id), symbol, new_val) {
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
    for depth in 0..self.scope_context.cf.stack.len() {
      let scope = self.scope_context.cf.get_mut_from_depth(depth);
      let deps = mem::take(&mut scope.deps);
      for dep in deps {
        self.consume(dep);
      }
    }
  }

  pub fn refer_to_diff_variable_scope(&mut self, another: ScopeId) {
    let target_depth = self.find_first_different_variable_scope(another);
    self.consume(self.get_assignment_deps(target_depth, ()));
  }
}
