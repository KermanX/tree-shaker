use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::ast::PrivateIdentifier;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_private_identifier(&mut self, node: &'a PrivateIdentifier<'a>) -> H::Entity {
    // FIXME: Not good
    self.exec_mangable_static_string(
      AstKind2::PrivateIdentifier(node),
      self.escape_private_identifier_name(node.name.as_str()),
    )
  }
}

