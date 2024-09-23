use super::cf_scope::CfScopes;
use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  entity::{dep::EntityDep, entity::Entity, literal::LiteralEntity, unknown::UnknownEntity},
};
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct VariableScope<'a> {
  pub dep: Option<EntityDep>,
  /// Cf scopes when the scope was created
  pub cf_scopes: CfScopes<'a>,
  /// (kind, is_consumed_exhaustively, entity_or_TDZ)
  pub variables: FxHashMap<SymbolId, (DeclarationKind, bool, Option<Entity<'a>>)>,
}

pub type VariableScopes<'a> = Vec<Rc<RefCell<VariableScope<'a>>>>;

impl<'a> VariableScope<'a> {
  pub fn new(dep: Option<EntityDep>, cf_scopes: CfScopes<'a>) -> Self {
    Self { dep, cf_scopes, variables: FxHashMap::default() }
  }

  pub fn declare(
    &mut self,
    analyzer: &mut Analyzer<'a>,
    kind: DeclarationKind,
    symbol: SymbolId,
    value: Option<Entity<'a>>,
  ) {
    if kind.is_redeclarable() {
      if let Some((old_kind, old_consumed, old_val)) = self.variables.get(&symbol) {
        if !old_kind.is_redeclarable() {
          // TODO: ERROR: "Variable already declared"
          analyzer.may_throw();
          value.map(|value| value.consume(analyzer));
          self.variables.insert(symbol, (kind, true, Some(UnknownEntity::new_unknown())));
        } else {
          if *old_consumed {
            value.map(|value| value.consume(analyzer));
          } else {
            self.variables.insert(symbol, (kind, false, value.or(old_val.clone())));
          }
        }
      } else {
        self.variables.insert(symbol, (kind, false, value));
      }
    } else {
      let old = self.variables.insert(symbol, (kind, false, value));
      if old.is_some() {
        // TODO: error "Variable already declared"
        analyzer.may_throw();
      }
    }
  }

  pub fn init(&mut self, analyzer: &mut Analyzer<'a>, symbol: SymbolId, value: Entity<'a>) {
    let (_, consumed, val) = self.variables.get_mut(&symbol).unwrap();
    if *consumed {
      value.consume(analyzer);
    } else {
      *val = Some(value);
    }
  }

  /// Returns (consumed, {None => TDZ, Some => value})
  pub fn read(&self, analyzer: &mut Analyzer<'a>, symbol: &SymbolId) -> (bool, Option<Entity<'a>>) {
    let (kind, consumed, value) = self.variables.get(symbol).unwrap();
    let value = value.as_ref().map_or_else(
      || {
        if kind.is_var() {
          Some(LiteralEntity::new_undefined())
        } else {
          // TODO: throw TDZ error
          analyzer.may_throw();
          analyzer.refer_global();
          None
        }
      },
      |v| Some(v.clone()),
    );
    (*consumed, value)
  }

  pub fn write(
    &mut self,
    analyzer: &mut Analyzer<'a>,
    symbol: SymbolId,
    (consumed, value): (bool, Entity<'a>),
  ) {
    let old = self.variables.get_mut(&symbol).unwrap();
    if !old.0.is_var() && old.2.is_none() {
      // TODO: throw TDZ error
      analyzer.may_throw();
      value.consume(analyzer);
    } else if old.0.is_const() {
      // TODO: throw error
      analyzer.may_throw();
      value.consume(analyzer);
    } else if old.1 {
      value.consume(analyzer);
    }
    *old = (old.0, consumed || old.1, Some(value));
  }
}
