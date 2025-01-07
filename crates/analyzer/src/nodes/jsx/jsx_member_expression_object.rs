use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::{Expression, JSXMemberExpressionObject};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_member_expression_object(
    &mut self,
    node: &'a JSXMemberExpressionObject<'a>,
  ) -> H::Entity {
    match node {
      JSXMemberExpressionObject::IdentifierReference(node) => {
        self.exec_identifier_reference_read(node)
      }
      JSXMemberExpressionObject::MemberExpression(node) => self.exec_jsx_member_expression(node),
      JSXMemberExpressionObject::ThisExpression(node) => self.exec_this_expression(node),
    }
  }
}
