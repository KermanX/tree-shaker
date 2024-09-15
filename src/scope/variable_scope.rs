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
  /// (is_consumed_exhaustively, entity)
  pub variables: FxHashMap<SymbolId, (bool, Option<Entity<'a>>)>,
}

pub type VariableScopes<'a> = Vec<Rc<RefCell<VariableScope<'a>>>>;

impl<'a> VariableScope<'a> {
  pub fn new(dep: Option<EntityDep>, cf_scopes: CfScopes<'a>) -> Self {
    Self { dep, cf_scopes, variables: FxHashMap::default() }
  }

  pub fn declare(&mut self, kind: DeclarationKind, symbol: SymbolId, value: Option<Entity<'a>>) {
    if kind.is_var() {
      let old = self.variables.get(&symbol);
      let new = match old {
        Some(old @ (true, _)) => old.clone(),
        _ => (false, Some(value.unwrap_or_else(LiteralEntity::new_undefined))),
      };
      self.variables.insert(symbol, new);
    } else {
      let old = self.variables.insert(symbol, (false, value));
      if old.is_some() && !kind.allow_override_var() {
        // TODO: error "Variable already declared"
      }
    }
  }

  pub fn init(&mut self, symbol: SymbolId, value: Entity<'a>) {
    let old = self.variables.get_mut(&symbol).unwrap();
    old.1 = Some(value);
  }

  pub fn read(&self, analyzer: &mut Analyzer<'a>, symbol: &SymbolId) -> (bool, Entity<'a>) {
    let (consumed, value) = self.variables.get(symbol).unwrap();
    let value = value.as_ref().map_or_else(
      || {
        // TODO: throw TDZ error
        analyzer.may_throw();
        UnknownEntity::new_unknown()
      },
      Entity::clone,
    );
    (*consumed, value)
  }

  pub fn write(&mut self, symbol: SymbolId, (consumed, value): (bool, Entity<'a>)) {
    let old = self.variables.get_mut(&symbol).unwrap();
    if old.1.is_none() {
      // TODO: throw TDZ error
    }
    *old = (consumed, Some(value));
  }
}
