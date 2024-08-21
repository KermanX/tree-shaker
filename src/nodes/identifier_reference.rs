use crate::{entity::Entity, TreeShaker};
use oxc::ast::ast::IdentifierReference;

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_identifier_reference_read(&mut self, node: &'a IdentifierReference) -> Entity {
    let reference = self.sematic.symbols().get_reference(node.reference_id().unwrap());
    assert!(reference.is_read());
    let symbol_id = reference.symbol_id();
    if let Some(symbol_id) = symbol_id {
      self.calc_symbol(symbol_id)
    } else {
      // TODO: Handle globals
      Entity::Unknown
    }
  }
}
