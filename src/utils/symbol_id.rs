use crate::{
  analyzer::Analyzer,
  entity::{Entity, LiteralEntity},
};
use oxc::{index::Idx, semantic::SymbolId};

impl<'a> Analyzer<'a> {
  pub fn serialize_internal_symbol_id(&self, symbol_id: SymbolId) -> Entity<'a> {
    self.factory.string(self.allocator.alloc(format!("__#symbol__{}", symbol_id.index())))
  }

  pub fn parse_internal_symbol_id(&self, entity: Entity<'a>) -> Option<SymbolId> {
    let literal = entity.get_literal(self)?;
    let LiteralEntity::String(string) = literal else { return None };
    if string.starts_with("__#symbol__") {
      string["__#symbol__".len()..].parse().ok().map(SymbolId::from_usize)
    } else {
      None
    }
  }
}
