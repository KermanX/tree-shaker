use super::exhaustive::TrackerRunner;
use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  consumable::{box_consumable, Consumable},
  entity::{Entity, UNDEFINED_ENTITY},
};
use oxc::semantic::{ScopeId, SymbolId};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{fmt, mem};

#[derive(Debug)]
pub struct Variable<'a> {
  pub kind: DeclarationKind,
  pub cf_scope: ScopeId,
  pub exhausted: bool,
  pub value: Option<Entity<'a>>,
  pub decl_dep: Consumable<'a>,
}

/// It's not good to clone, but it's fine for now
impl Clone for Variable<'_> {
  fn clone(&self) -> Self {
    Self {
      kind: self.kind,
      cf_scope: self.cf_scope,
      exhausted: self.exhausted,
      value: self.value.clone(),
      decl_dep: self.decl_dep.cloned(),
    }
  }
}

pub struct VariableScope<'a> {
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
  pub fn new() -> Self {
    Self { variables: Default::default(), exhaustive_deps: Default::default() }
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
          self.write_on_scope(id, symbol, new_val);
        }
      } else {
        // Re-declaration
      }
    } else {
      let has_fn_value = fn_value.is_some();
      let variable = Variable {
        kind,
        cf_scope: if kind.is_var() {
          self.cf_scope_id_of_call_scope()
        } else {
          self.scope_context.cf.current_id()
        },
        exhausted: false,
        value: fn_value,
        decl_dep,
      };
      self.scope_context.variable.get_mut(id).variables.insert(symbol, variable);
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

    if variable.kind.is_redeclarable() {
      if let Some(value) = value {
        self.write_on_scope(id, symbol, self.factory.computed(value, init_dep));
      } else {
        // Do nothing
      }
    } else if variable.exhausted {
      if let Some(value) = value {
        self.consume(value);
      }
      self.consume(init_dep);
    } else {
      variable.value =
        Some(self.factory.computed(value.unwrap_or(self.factory.undefined), init_dep));
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
          .then(|| self.factory.computed(self.factory.undefined, variable.decl_dep.cloned()))
      });

      let target_cf_scope = self.find_first_different_cf_scope(variable.cf_scope);
      if !variable.exhausted {
        self.mark_exhaustive_read((id, symbol), target_cf_scope);
      }

      if value.is_none() {
        // TDZ
        self.consume(variable.decl_dep.cloned());
        self.handle_tdz(target_cf_scope);
      }

      value
    })
  }

  fn write_on_scope(&mut self, id: ScopeId, symbol: SymbolId, new_val: Entity<'a>) -> bool {
    if let Some(variable) = self.scope_context.variable.get(id).variables.get(&symbol).cloned() {
      if variable.kind.is_untracked() {
        self.consume(new_val);
      } else if variable.kind.is_const() {
        self.thrown_builtin_error("Cannot assign to const variable");
        self.consume(variable.decl_dep);
        new_val.consume(self);
      } else {
        let target_cf_scope = self.find_first_different_cf_scope(variable.cf_scope);
        let dep = (self.get_exec_dep(target_cf_scope), variable.decl_dep.cloned());

        if variable.exhausted {
          self.consume(dep);
          self.consume(new_val);
        } else {
          let old_val = variable.value;
          let (should_consume, indeterminate) = if old_val.is_some() {
            // Normal write
            self.mark_exhaustive_write((id, symbol), target_cf_scope)
          } else if !variable.kind.is_redeclarable() {
            // TDZ write
            self.handle_tdz(target_cf_scope);
            (true, false)
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
            let variable =
              self.scope_context.variable.get_mut(id).variables.get_mut(&symbol).unwrap();
            variable.value = Some(self.factory.computed(
              if indeterminate {
                self.factory.union(vec![
                  old_val.unwrap_or(unsafe { mem::transmute(UNDEFINED_ENTITY) }),
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
        variable.value = Some(self.factory.unknown);
      }
      true
    } else {
      false
    }
  }

  fn mark_untracked_on_scope(&mut self, symbol: SymbolId) {
    let cf_scope_depth = self.call_scope().cf_scope_depth;
    let variable = Variable {
      exhausted: true,
      kind: DeclarationKind::UntrackedVar,
      cf_scope: self.scope_context.cf.stack[cf_scope_depth],
      value: Some(self.factory.unknown),
      decl_dep: box_consumable(()),
    };
    let old = self.variable_scope_mut().variables.insert(symbol, variable);
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

    let variable_scope = self.scope_context.variable.current_id();
    self.declare_on_scope(variable_scope, kind, symbol, decl_dep, fn_value);
  }

  pub fn init_symbol(
    &mut self,
    symbol: SymbolId,
    value: Option<Entity<'a>>,
    init_dep: Consumable<'a>,
  ) {
    let variable_scope = self.scope_context.variable.current_id();
    self.init_on_scope(variable_scope, symbol, value, init_dep);
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
      if self.write_on_scope(id, symbol, new_val) {
        return;
      }
    }
    self.consume(new_val);
    self.mark_unresolved_reference(symbol);
  }

  fn mark_unresolved_reference(&mut self, symbol: SymbolId) {
    if self.semantic.symbols().get_flags(symbol).is_function_scoped_declaration() {
      self.insert_var_decl(symbol);
      self.mark_untracked_on_scope(symbol);
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
    self.refer_to_global();
  }
}
