use super::cf_scope::CfScopes;
use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  entity::{Consumable, Entity, LiteralEntity, UnknownEntity},
};
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct VariableScope<'a> {
  pub dep: Option<Consumable<'a>>,
  /// Cf scopes when the scope was created
  pub cf_scopes: CfScopes<'a>,
  /// (kind, is_consumed_exhaustively, entity_or_TDZ)
  pub variables: RefCell<FxHashMap<SymbolId, (DeclarationKind, bool, Option<Entity<'a>>)>>,
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
    value: Option<Entity<'a>>,
  ) {
    let mut variables = self.variables.borrow_mut();
    if kind.is_redeclarable() {
      if let Some((old_kind, old_consumed, old_val)) = variables.get(&symbol).cloned() {
        if !old_kind.is_redeclarable() {
          // TODO: ERROR: "Variable already declared"
          analyzer.explicit_throw_unknown();
          value.map(|value| value.consume(analyzer));
          variables.insert(symbol, (kind, true, Some(UnknownEntity::new_unknown())));
        } else {
          if old_consumed {
            value.map(|value| value.consume(analyzer));
          } else {
            variables.insert(symbol, (kind, false, value.or(old_val.clone())));
          }
        }
      } else {
        variables.insert(symbol, (kind, false, value));
      }
    } else {
      let old = variables.insert(symbol, (kind, false, value));
      if old.is_some() {
        // TODO: error "Variable already declared"
        analyzer.explicit_throw_unknown();
      }
    }
  }

  pub fn init(&self, analyzer: &mut Analyzer<'a>, symbol: SymbolId, value: Entity<'a>) {
    let mut variables = self.variables.borrow_mut();
    let (_, consumed, val) = variables.get_mut(&symbol).unwrap();
    if *consumed {
      value.consume(analyzer);
    } else {
      *val = Some(value);
    }
  }

  /// Returns (consumed, {None => TDZ, Some => value})
  pub fn read(&self, symbol: &SymbolId) -> (bool, Option<Entity<'a>>) {
    let variables = self.variables.borrow();
    let (kind, consumed, value) = variables.get(symbol).unwrap();
    let value = value.as_ref().map_or_else(
      || {
        if kind.is_var() {
          Some(LiteralEntity::new_undefined())
        } else {
          None
        }
      },
      |v| Some(v.clone()),
    );
    (*consumed, value)
  }

  pub fn write(
    &self,
    analyzer: &mut Analyzer<'a>,
    symbol: SymbolId,
    (consumed, value): (bool, Entity<'a>),
  ) {
    let mut variables = self.variables.borrow_mut();
    let old = variables.get_mut(&symbol).unwrap();
    if !old.0.is_var() && old.2.is_none() {
      // TODO: throw TDZ error
      analyzer.may_throw();
      value.consume(analyzer);
    } else if old.0.is_const() {
      // TODO: throw error
      analyzer.explicit_throw_unknown();
      value.consume(analyzer);
    } else if old.1 {
      value.consume(analyzer);
    }
    *old = (old.0, consumed || old.1, Some(value));
  }

  pub fn consume(&self, analyzer: &mut Analyzer<'a>, symbol: SymbolId) {
    if let (false, Some(val)) = self.read(&symbol) {
      val.consume(analyzer);
      self.write(analyzer, symbol, (true, UnknownEntity::new_unknown()));
    }
  }
}
