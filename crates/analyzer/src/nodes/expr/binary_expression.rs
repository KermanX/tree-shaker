use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::BinaryExpression;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_binary_expression(&mut self, node: &'a BinaryExpression<'a>) -> H::Entity {
    let lhs = self.exec_expression(&node.left);
    let rhs = self.exec_expression(&node.right);

    self.host.binary_op(node.operator, lhs, rhs)
  }
}
