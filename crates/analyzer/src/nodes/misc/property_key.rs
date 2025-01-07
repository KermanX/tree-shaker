use crate::{host::Host, analyzer::Analyzer};
use oxc::{ast::ast::PropertyKey, span::GetSpan};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_property_key(&mut self, node: &'a PropertyKey<'a>) -> H::Entity {
    match node {
      PropertyKey::StaticIdentifier(node) => self.exec_identifier_name(node),
      PropertyKey::PrivateIdentifier(node) => self.exec_private_identifier(node),
      node => self.exec_expression(node.to_expression()).get_to_property_key(self),
    }
  }
}

