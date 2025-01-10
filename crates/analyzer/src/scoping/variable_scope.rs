use crate::{
  ast::{AstKind2, DeclarationKind},
  EcmaAnalyzer,
};

use super::exhaustive::ExhaustiveCallback;
use oxc::semantic::{ScopeId, SymbolId};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{cell::RefCell, fmt};

#[derive(Debug)]
pub struct Variable<'a, A: EcmaAnalyzer<'a> + ?Sized> {
  pub kind: DeclarationKind,
  pub cf_scope: ScopeId,
  pub exhausted: Option<LazyConsumable<'a>>,
  pub value: Option<A::Entity>,
  pub decl_node: AstKind2<'a>,
}

#[derive(Default)]
pub struct VariableScope<'a, A: EcmaAnalyzer<'a> + ?Sized> {
  pub variables: FxHashMap<SymbolId, &'a RefCell<Variable<'a, A>>>,
  pub this: Option<A::Entity>,
  pub arguments: Option<(A::Entity, Vec<SymbolId>)>,
  pub exhaustive_callbacks: FxHashMap<SymbolId, FxHashSet<ExhaustiveCallback<'a, A>>>,
}

impl<'a, A: EcmaAnalyzer<'a> + ?Sized> fmt::Debug for VariableScope<'a, A> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut map = f.debug_map();
    for (k, v) in self.variables.iter() {
      let v = v.borrow();
      map.entry(&k, &format!("{:?} {}", v.kind, v.value.is_some()));
    }
    map.finish()
  }
}

impl<'a, A: EcmaAnalyzer<'a> + ?Sized> VariableScope<'a, A> {
  pub fn new() -> Self {
    Self::default()
  }
}

pub trait VariableScopeAnalyzer<'a> {
  type VariableExtra: fmt::Debug;

  fn declare_on_scope(
    &mut self,
    id: ScopeId,
    kind: DeclarationKind,
    symbol: SymbolId,
    decl_node: AstKind2<'a>,
    fn_value: Option<Self::Entity>,
  ) where
    Self: EcmaAnalyzer<'a>,
  {
    if let Some(variable) = self.scoping().variable.get(id).variables.get(&symbol) {
      // Here we can't use kind.is_untracked() because this time we are declaring a variable
      let old_kind = variable.borrow().kind;

      if old_kind.is_untracked() {
        self.consume(decl_node);
        if let Some(val) = fn_value {
          val.consume(self)
        }
        return;
      }

      if old_kind.is_shadowable() && kind.is_redeclarable() {
        // Redeclaration is sometimes allowed
        // var x = 1; var x = 2;
        // function f(x) { var x }
        let mut variable = variable.borrow_mut();
        variable.kind = kind;
        // FIXME: Not sure if this is correct - how to handle the first declaration?
        variable.decl_node = decl_node;
        drop(variable);
        if let Some(new_val) = fn_value {
          self.write_on_scope(id, symbol, new_val);
        }
      } else {
        // Re-declaration
      }
    } else {
      let has_fn_value = fn_value.is_some();
      let variable = self.allocator.alloc(RefCell::new(Variable {
        kind,
        cf_scope: if kind.is_var() {
          self.cf_scope_id_of_call_scope()
        } else {
          self.scoping().cf.current_id()
        },
        exhausted: None,
        value: fn_value,
        decl_node,
      }));
      self.scoping().variable.get_mut(id).variables.insert(symbol, variable);
      if has_fn_value {
        self.add_exhaustive_callbacks(false, (id, symbol));
      }
    }
  }

  fn init_on_scope(
    &mut self,
    id: ScopeId,
    symbol: SymbolId,
    value: Option<Self::Entity>,
    init_node: AstKind2<'a>,
  ) where
    Self: EcmaAnalyzer<'a>,
  {
    let variable = self.scoping().variable.get_mut(id).variables.get_mut(&symbol).unwrap();

    let variable_ref = variable.borrow();
    if variable_ref.kind.is_redeclarable() {
      if let Some(value) = value {
        drop(variable_ref);
        self.write_on_scope(id, symbol, self.factory.computed(value, init_node));
      } else {
        // Do nothing
      }
    } else if let Some(deps) = variable_ref.exhausted {
      deps.push(self, self.consumable((init_node, value)));
    } else {
      drop(variable_ref);
      variable.borrow_mut().value =
        Some(self.factory.computed(value.unwrap_or(self.factory.undefined), init_node));
      self.add_exhaustive_callbacks(false, (id, symbol));
    }
  }

  /// None: not in this scope
  /// Some(None): in this scope, but TDZ
  /// Some(Some(val)): in this scope, and val is the value
  fn read_on_scope(&mut self, id: ScopeId, symbol: SymbolId) -> Option<Option<Self::Entity>>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping().variable.get(id).variables.get(&symbol).copied().map(|variable| {
      let variable_ref = variable.borrow();
      let value = variable_ref.value.or_else(|| {
        variable_ref
          .kind
          .is_var()
          .then(|| self.factory.computed(self.factory.undefined, variable_ref.decl_node))
      });

      let value = if let Some(dep) = variable_ref.exhausted {
        drop(variable_ref);
        if let Some(value) = value {
          Some(self.factory.computed(value, dep))
        } else {
          self.consume(dep);
          None
        }
      } else {
        let target_cf_scope = self.find_first_different_cf_scope(variable_ref.cf_scope);
        drop(variable_ref);
        self.mark_exhaustive_read((id, symbol), target_cf_scope);
        value
      };

      if value.is_none() {
        // TDZ
        let variable_ref = variable.borrow();
        self.consume(variable_ref.decl_node);
        let target_cf_scope = self.find_first_different_cf_scope(variable_ref.cf_scope);
        self.handle_tdz(target_cf_scope);
      }

      value
    })
  }

  fn write_on_scope(&mut self, id: ScopeId, symbol: SymbolId, new_val: Self::Entity) -> bool
  where
    Self: EcmaAnalyzer<'a>,
  {
    if let Some(variable) = self.scoping().variable.get(id).variables.get(&symbol).copied() {
      let kind = variable.borrow().kind;
      if kind.is_untracked() {
        self.consume(new_val);
      } else if kind.is_const() {
        self.thrown_builtin_error("Cannot assign to const variable");
        self.consume(variable.borrow().decl_node);
        new_val.consume(self);
      } else {
        let variable_ref = variable.borrow();
        let target_cf_scope = self.find_first_different_cf_scope(variable_ref.cf_scope);
        let dep = (self.get_exec_dep(target_cf_scope), variable_ref.decl_node);

        if let Some(deps) = variable_ref.exhausted {
          deps.push(self, self.consumable((dep, new_val)));
        } else {
          let old_val = variable_ref.value;
          let (should_consume, indeterminate) = if old_val.is_some() {
            // Normal write
            self.mark_exhaustive_write((id, symbol), target_cf_scope)
          } else if !variable_ref.kind.is_redeclarable() {
            // TDZ write
            self.handle_tdz(target_cf_scope);
            (true, false)
          } else {
            // Write uninitialized `var`
            self.mark_exhaustive_write((id, symbol), target_cf_scope)
          };
          drop(variable_ref);

          let mut variable_ref = variable.borrow_mut();
          if should_consume {
            variable_ref.exhausted =
              Some(self.factory.new_lazy_consumable(self.consumable((dep, new_val, old_val))));
            variable_ref.value = Some(self.factory.unknown());
          } else {
            variable_ref.value = Some(self.factory.computed(
              if indeterminate {
                self.factory.union((old_val.unwrap_or(self.factory.undefined), new_val))
              } else {
                new_val
              },
              self.consumable(dep),
            ));
          };
          drop(variable_ref);

          self.add_exhaustive_callbacks(should_consume, (id, symbol));
        }
      }
      true
    } else {
      false
    }
  }

  fn consume_on_scope(&mut self, id: ScopeId, symbol: SymbolId) -> bool
  where
    Self: EcmaAnalyzer<'a>,
  {
    if let Some(variable) = self.scoping().variable.get(id).variables.get(&symbol).copied() {
      let variable_ref = variable.borrow();
      if let Some(dep) = variable_ref.exhausted {
        drop(variable_ref);
        self.consume(dep);
      } else {
        self.consume(variable_ref.decl_node);
        if let Some(value) = &variable_ref.value {
          value.consume(self);
        }
        drop(variable_ref);

        let mut variable_ref = variable.borrow_mut();
        variable_ref.exhausted = Some(self.factory.consumed_lazy_consumable);
        variable_ref.value = Some(self.factory.unknown());
      }
      true
    } else {
      false
    }
  }

  fn mark_untracked_on_scope(&mut self, symbol: SymbolId)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let cf_scope_depth = self.call_scope().cf_scope_depth;
    let variable = self.allocator.alloc(RefCell::new(Variable {
      exhausted: Some(self.factory.consumed_lazy_consumable),
      kind: DeclarationKind::UntrackedVar,
      cf_scope: self.scoping().cf.stack[cf_scope_depth],
      value: Some(self.factory.unknown()),
      decl_node: AstKind2::Environment,
    }));
    let old = self.variable_scope_mut().variables.insert(symbol, variable);
    assert!(old.is_none());
  }

  fn consume_arguments_on_scope(&mut self, id: ScopeId) -> bool
  where
    Self: EcmaAnalyzer<'a>,
  {
    if let Some((args_entity, args_symbols)) = self.scoping().variable.get(id).arguments.clone() {
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

  fn declare_symbol(
    &mut self,
    symbol: SymbolId,
    decl_node: AstKind2<'a>,
    exporting: bool,
    kind: DeclarationKind,
    fn_value: Option<Self::Entity>,
  ) where
    Self: EcmaAnalyzer<'a>,
  {
    if exporting {
      self.named_exports.push(symbol);
    }
    if kind == DeclarationKind::FunctionParameter {
      if let Some(arguments) = &mut self.variable_scope_mut().arguments {
        arguments.1.push(symbol);
      }
    }

    let variable_scope = self.scoping().variable.current_id();
    self.declare_on_scope(variable_scope, kind, symbol, decl_node, fn_value);
  }

  fn init_symbol(&mut self, symbol: SymbolId, value: Option<Self::Entity>, init_node: AstKind2<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let variable_scope = self.scoping().variable.current_id();
    self.init_on_scope(variable_scope, symbol, value, init_node);
  }

  /// `None` for TDZ
  fn read_symbol(&mut self, symbol: SymbolId) -> Option<Self::Entity>
  where
    Self: EcmaAnalyzer<'a>,
  {
    for depth in (0..self.scoping().variable.stack.len()).rev() {
      let id = self.scoping().variable.stack[depth];
      if let Some(value) = self.read_on_scope(id, symbol) {
        return value;
      }
    }
    self.mark_unresolved_reference(symbol);
    Some(self.factory.unknown())
  }

  fn write_symbol(&mut self, symbol: SymbolId, new_val: Self::Entity)
  where
    Self: EcmaAnalyzer<'a>,
  {
    for depth in (0..self.scoping().variable.stack.len()).rev() {
      let id = self.scoping().variable.stack[depth];
      if self.write_on_scope(id, symbol, new_val) {
        return;
      }
    }
    self.consume(new_val);
    self.mark_unresolved_reference(symbol);
  }

  fn mark_unresolved_reference(&mut self, symbol: SymbolId)
  where
    Self: EcmaAnalyzer<'a>,
  {
    if self.semantic.symbols().get_flags(symbol).is_function_scoped_declaration() {
      self.mark_untracked_on_scope(symbol);
    } else {
      self.thrown_builtin_error("Unresolved identifier reference");
    }
  }

  fn handle_tdz(&mut self, target_cf_scope: usize)
  where
    Self: EcmaAnalyzer<'a>,
  {
    if self.has_exhaustive_scope_since(target_cf_scope) {
      self.may_throw();
    } else {
      self.thrown_builtin_error("Cannot access variable before initialization");
    }
    self.refer_to_global();
  }

  fn get_this(&self) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>,
  {
    for depth in (0..self.scoping().variable.stack.len()).rev() {
      let scope = self.scoping().variable.get_from_depth(depth);
      if let Some(this) = scope.this {
        return this;
      }
    }
    unreachable!()
  }
}
