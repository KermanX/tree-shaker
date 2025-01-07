use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::ast::{Expression, JSXElementName};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_element_name(&mut self, node: &'a JSXElementName<'a>) -> H::Entity {
    match node {
      JSXElementName::Identifier(_node) => self.factory.unknown_string,
      JSXElementName::IdentifierReference(node) => self.exec_identifier_reference_read(node),
      JSXElementName::NamespacedName(_node) => self.factory.unknown_string,
      JSXElementName::MemberExpression(node) => self.exec_jsx_member_expression(node),
      JSXElementName::ThisExpression(node) => self.exec_this_expression(node),
    }
  }
}

