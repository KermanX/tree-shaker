use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::ast::{Expression, PrivateInExpression};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_private_in_expression(&mut self, node: &'a PrivateInExpression<'a>) -> H::Entity {
    let right = self.exec_expression(&node.right);
    self.factory.computed_unknown_boolean(right)
  }
}

