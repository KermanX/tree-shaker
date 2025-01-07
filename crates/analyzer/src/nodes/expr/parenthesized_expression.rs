use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::ast::{Expression, ParenthesizedExpression};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_parenthesized_expression(
    &mut self,
    node: &'a ParenthesizedExpression<'a>,
  ) -> H::Entity {
    self.exec_expression(&node.expression)
  }
}

