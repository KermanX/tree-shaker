use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::IdentifierName;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_identifier_name(&mut self, node: &'a IdentifierName<'a>) -> H::Entity {
    self.exec_mangable_static_string(AstKind2::IdentifierName(node), node.name.as_str())
  }
}
