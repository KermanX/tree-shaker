use super::cf_scope::CfScopes;
use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  entity::{Consumable, Entity, ForwardedEntity, LiteralEntity, UnionEntity, UnknownEntity},
};
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Debug)]
pub struct Variable<'a> {
  pub kind: DeclarationKind,
  pub exhausted: bool,
  pub value: Option<Entity<'a>>,
  pub decl_dep: Consumable<'a>,
}

pub struct VariableScope<'a> {
  pub dep: Option<Consumable<'a>>,
  /// Cf scopes when the scope was created
  pub cf_scopes: CfScopes<'a>,
  pub variables: RefCell<FxHashMap<SymbolId, Variable<'a>>>,
}

impl fmt::Debug for VariableScope<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut map = f.debug_map();
    for (k, v) in self.variables.borrow().iter() {
      map.entry(&k, &format!("{:?}", v.kind));
    }
    map.finish()
  }
}

pub type VariableScopes<'a> = Vec<Rc<VariableScope<'a>>>;

impl<'a> VariableScope<'a> {
  pub fn new(dep: Option<Consumable<'a>>, cf_scopes: CfScopes<'a>) -> Self {
    Self { dep, cf_scopes, variables: Default::default() }
  }

  pub fn declare(
    &self,
    analyzer: &mut Analyzer<'a>,
    kind: DeclarationKind,
    symbol: SymbolId,
    decl_dep: Consumable<'a>,
    fn_value: Option<Entity<'a>>,
  ) {
    let mut variables = self.variables.borrow_mut();
    let old = variables.get(&symbol);

    if let Some(old) = old.and_then(|old| (!old.kind.is_shadowable()).then_some(old)) {
      // Here we can't use kind.is_untracked() because this time we are declaring a variable
      if old.kind.is_untracked() {
        analyzer.consume(decl_dep);
        fn_value.map(|val| val.consume(analyzer));
        return;
      }

      if old.kind.is_var() && kind.is_var() {
        // var x = 1; var x = 2;
        todo!();
      } else {
        let name = analyzer.semantic.symbols().get_name(symbol);
        analyzer.explicit_throw_unknown(format!("Variable {name:?} already declared"));
        analyzer.consume_ref(&old.decl_dep);
        analyzer.consume(decl_dep);
      }
    } else {
      variables.insert(symbol, Variable { kind, exhausted: false, value: fn_value, decl_dep });
    }
  }

  pub fn init(&self, analyzer: &mut Analyzer<'a>, symbol: SymbolId, value: Option<Entity<'a>>) {
    let mut variables = self.variables.borrow_mut();
    let variable = variables.get_mut(&symbol).unwrap();

    if variable.kind.is_untracked() {
      if let Some(value) = value {
        analyzer.consume(value);
      }
    } else if variable.kind.is_var() {
      if let Some(value) = value {
        self.write(analyzer, symbol, &value, analyzer.scope_context.variable_scopes.len() - 1);
      } else {
        // Do nothing
      }
    } else {
      debug_assert!(!variable.exhausted);
      variable.value = Some(value.unwrap_or_else(LiteralEntity::new_undefined));
    }
  }

  /// None: not in this scope
  /// Some(None): in this scope, but TDZ
  /// Some(Some(val)): in this scope, and val is the value
  pub fn read(&self, analyzer: &mut Analyzer<'a>, symbol: SymbolId) -> Option<Option<Entity<'a>>> {
    let variables = self.variables.borrow();
    variables.get(&symbol).map(|variable| {
      let value = variable
        .value
        .clone()
        .or_else(|| variable.kind.is_var().then(LiteralEntity::new_undefined));

      let target_cf_scope = analyzer.find_first_different_cf_scope(&self.cf_scopes);
      if let Some(value) = value {
        analyzer.mark_exhaustive_read(&value, symbol, target_cf_scope);
        Some(ForwardedEntity::new(value, variable.decl_dep.clone()))
      } else {
        analyzer.consume(variable.decl_dep.clone());
        analyzer.handle_tdz(target_cf_scope);
        None
      }
    })
  }

  pub fn write(
    &self,
    analyzer: &mut Analyzer<'a>,
    symbol: SymbolId,
    new_val: &Entity<'a>,
    self_index: usize,
  ) -> bool {
    let mut variables = self.variables.borrow_mut();
    if let Some(variable) = variables.get_mut(&symbol) {
      let new_val = new_val.clone();
      if variable.kind.is_untracked() {
        analyzer.consume(new_val);
      } else if variable.kind.is_const() {
        analyzer.explicit_throw_unknown("Cannot assign to const variable");
        new_val.consume(analyzer);
      } else {
        let target_cf_scope = analyzer.find_first_different_cf_scope(&self.cf_scopes);
        let dep = analyzer.get_assignment_deps(self_index, variable.decl_dep.clone());

        if variable.exhausted {
          analyzer.consume(dep);
          analyzer.consume(new_val);
        } else {
          let old_val = variable.value.clone();
          let should_consume = if let Some(old_val) = &old_val {
            analyzer.mark_exhaustive_write(old_val, symbol, target_cf_scope)
          } else if !variable.kind.is_var() {
            analyzer.handle_tdz(target_cf_scope);
            true
          } else {
            // Uninitialized `var`
            false
          };

          if should_consume {
            analyzer.consume(dep);
            analyzer.consume(new_val);
            if let Some(old_val) = &old_val {
              old_val.consume(analyzer);
            }

            variable.exhausted = true;
            variable.value = Some(UnknownEntity::new_unknown());
          } else {
            let indeterminate =
              analyzer.is_relatively_indeterminate(target_cf_scope, &self.cf_scopes);

            variable.value = Some(ForwardedEntity::new(
              if indeterminate {
                UnionEntity::new(vec![old_val.unwrap(), new_val])
              } else {
                new_val
              },
              dep,
            ));
          };

          analyzer.exec_exhaustive_deps(should_consume, symbol);
        }
      }
      true
    } else {
      false
    }
  }

  pub fn consume(&self, analyzer: &mut Analyzer<'a>, symbol: SymbolId) -> bool {
    let mut variables = self.variables.borrow_mut();
    if let Some(variable) = variables.get_mut(&symbol) {
      if !variable.exhausted {
        variable.decl_dep.consume(analyzer);
        if let Some(value) = &variable.value {
          value.consume(analyzer);
          variable.exhausted = true;
          variable.value = Some(UnknownEntity::new_unknown());
        }
        variable.exhausted = true;
      }
      true
    } else {
      false
    }
  }

  pub fn mark_untracked(&self, symbol: SymbolId) {
    let mut variables = self.variables.borrow_mut();
    let old = variables.insert(
      symbol,
      Variable {
        exhausted: true,
        kind: DeclarationKind::UntrackedVar,
        value: None,
        decl_dep: ().into(),
      },
    );
    debug_assert!(old.is_none());
  }
}
