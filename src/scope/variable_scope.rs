use crate::ast::DeclarationKind;
use crate::entity::entity::Entity;
use crate::entity::literal::LiteralEntity;
use crate::entity::unknown::UnknownEntity;
use oxc::semantic::ScopeId;
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use super::cf_scope::CfScopes;

#[derive(Debug)]
pub struct VariableScope<'a> {
  pub id: ScopeId,
  /// Cf scopes when the scope was created
  pub cf_scopes: CfScopes<'a>,
  /// (is_consumed_exhaustively, entity)
  pub variables: FxHashMap<SymbolId, (bool, Option<Entity<'a>>)>,
}

pub type VariableScopes<'a> = Vec<Rc<RefCell<VariableScope<'a>>>>;

static VARIABLE_SCOPE_ID: AtomicU32 = AtomicU32::new(0);

impl<'a> VariableScope<'a> {
  pub fn new(cf_scopes: CfScopes<'a>) -> Self {
    Self {
      id: ScopeId::new(VARIABLE_SCOPE_ID.fetch_add(1, Ordering::Relaxed)),
      cf_scopes,
      variables: FxHashMap::default(),
    }
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

  pub fn read(&self, symbol: &SymbolId) -> (bool, Entity<'a>) {
    let (consumed, value) = self.variables.get(symbol).unwrap();
    let value = value.as_ref().map_or_else(
      || {
        // TODO: throw TDZ error
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
