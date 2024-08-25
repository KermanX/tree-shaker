use crate::entity::entity::Entity;
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub(crate) struct VariableScope<'a> {
  pub variables: FxHashMap<SymbolId, Entity<'a>>,
}

impl<'a> VariableScope<'a> {
  pub fn new() -> Self {
    Self { variables: FxHashMap::default() }
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
}
