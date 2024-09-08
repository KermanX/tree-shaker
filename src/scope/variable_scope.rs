use crate::entity::entity::Entity;
use oxc::semantic::ScopeId;
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub struct VariableScope<'a> {
  pub id: ScopeId,
  pub has_effect: bool,
  pub variables: FxHashMap<SymbolId, Entity<'a>>,
  pub cf_scope_id: ScopeId,
}

static FUNCTION_SCOPE_ID: AtomicU32 = AtomicU32::new(0);

impl<'a> VariableScope<'a> {
  pub fn new(cf_scope_id: ScopeId) -> Self {
    Self {
      id: ScopeId::new(FUNCTION_SCOPE_ID.fetch_add(1, Ordering::Relaxed)),
      has_effect: false,
      variables: FxHashMap::default(),
      cf_scope_id,
    }
  }

  pub fn set(&mut self, symbol: SymbolId, entity: Entity<'a>) -> Option<Entity<'a>> {
    self.variables.insert(symbol, entity)
  }

  pub fn declare(&mut self, symbol: SymbolId, entity: Entity<'a>) {
    assert!(self.set(symbol, entity).is_none());
  }

  pub fn get(&self, symbol: &SymbolId) -> Option<&Entity<'a>> {
    self.variables.get(symbol)
  }

  pub fn has(&self, symbol: &SymbolId) -> bool {
    self.variables.contains_key(symbol)
  }
}
