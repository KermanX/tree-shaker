use super::exhaustive::TrackerRunner;
use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  consumable::{box_consumable, Consumable, ConsumableVec},
  entity::{Entity, UNDEFINED_ENTITY},
};
use oxc::semantic::{ScopeId, SymbolId};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{cell::RefCell, fmt, mem, rc::Rc};

#[derive(Debug)]
pub struct Variable<'a> {
  pub kind: DeclarationKind,
  pub cf_scope: ScopeId,
  pub exhausted: Option<Rc<RefCell<ConsumableVec<'a>>>>,
  pub value: Option<Entity<'a>>,
  pub decl_dep: Consumable<'a>,
}

/// It's not good to clone, but it's fine for now
impl Clone for Variable<'_> {
  fn clone(&self) -> Self {
    Self {
      kind: self.kind,
      cf_scope: self.cf_scope,
      exhausted: self.exhausted.clone(),
      value: self.value.clone(),
      decl_dep: self.decl_dep.cloned(),
    }
  }
}

#[derive(Default)]
pub struct VariableScope<'a> {
  pub variables: FxHashMap<SymbolId, Variable<'a>>,
  pub this: Option<Entity<'a>>,
  pub arguments: Option<(Entity<'a>, Vec<SymbolId>)>,
  pub super_class: Option<Entity<'a>>,
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
    Self::default()
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
        exhausted: None,
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
    } else if let Some(deps) = variable.exhausted.clone() {
      deps.borrow_mut().push(box_consumable((init_dep, value)));
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

      let value = if let Some(dep) = variable.exhausted.clone() {
        if let Some(value) = value {
          Some(self.factory.computed(value, dep))
        } else {
          self.consume(dep);
          None
        }
      } else {
        let target_cf_scope = self.find_first_different_cf_scope(variable.cf_scope);
        self.mark_exhaustive_read((id, symbol), target_cf_scope);
        value
      };

      if value.is_none() {
        // TDZ
        self.consume(variable.decl_dep.cloned());
        let target_cf_scope = self.find_first_different_cf_scope(variable.cf_scope);
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

        if let Some(deps) = variable.exhausted {
          deps.borrow_mut().push(box_consumable((dep, new_val)));
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
            let variable =
              self.scope_context.variable.get_mut(id).variables.get_mut(&symbol).unwrap();
            variable.exhausted =
              Some(Rc::new(RefCell::new(vec![box_consumable((dep, new_val, old_val))])));
            variable.value = Some(self.factory.unknown());
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
      if let Some(dep) = variable.exhausted {
        self.consume(dep);
      } else {
        variable.decl_dep.consume(self);
        if let Some(value) = &variable.value {
          value.consume(self);
        }
        let variable = self.scope_context.variable.get_mut(id).variables.get_mut(&symbol).unwrap();
        variable.exhausted = Some(Default::default());
        variable.value = Some(self.factory.unknown());
      }
      true
    } else {
      false
    }
  }

  fn mark_untracked_on_scope(&mut self, symbol: SymbolId) {
    let cf_scope_depth = self.call_scope().cf_scope_depth;
    let variable = Variable {
      exhausted: Some(Default::default()),
      kind: DeclarationKind::UntrackedVar,
      cf_scope: self.scope_context.cf.stack[cf_scope_depth],
      value: Some(self.factory.unknown()),
      decl_dep: box_consumable(()),
    };
    let old = self.variable_scope_mut().variables.insert(symbol, variable);
    debug_assert!(old.is_none());
  }

  pub fn consume_arguments_on_scope(&mut self, id: ScopeId) -> bool {
    if let Some((args_entity, args_symbols)) = self.scope_context.variable.get(id).arguments.clone()
    {
      args_entity.consume(self);
      let mut arguments_consumed = true;
      for symbol in args_symbols {
        if !self.consume_on_scope(id, symbol) {
          // Still inside parameter declaration
          arguments_consumed = false;
        }
      }
      arguments_consumed
    } else {
      true
    }
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
      if let Some(arguments) = &mut self.variable_scope_mut().arguments {
        arguments.1.push(symbol);
      }
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
    Some(self.factory.unknown())
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
      self.mark_untracked_on_scope(symbol);
    } else {
      self.thrown_builtin_error("Unresolved identifier reference");
    }
  }

  pub fn handle_tdz(&mut self, target_cf_scope: usize) {
    if self.has_exhaustive_scope_since(target_cf_scope) {
      self.may_throw();
    } else {
      self.thrown_builtin_error("Cannot access variable before initialization");
    }
    self.refer_to_global();
  }

  pub fn get_this(&self) -> Entity<'a> {
    for depth in (0..self.scope_context.variable.stack.len()).rev() {
      let scope = self.scope_context.variable.get_from_depth(depth);
      if let Some(this) = scope.this {
        return this;
      }
    }
    unreachable!()
  }

  pub fn get_super(&self) -> Entity<'a> {
    for depth in (0..self.scope_context.variable.stack.len()).rev() {
      let scope = self.scope_context.variable.get_from_depth(depth);
      if let Some(super_class) = scope.super_class {
        return super_class;
      }
    }
    self.factory.unknown()
  }
}
