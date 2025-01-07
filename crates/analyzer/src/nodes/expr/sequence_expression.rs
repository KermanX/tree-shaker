use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  ast::ast::{Expression, SequenceExpression},
  span::SPAN,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_sequence_expression(&mut self, node: &'a SequenceExpression<'a>) -> H::Entity {
    let mut last = None;
    for expression in &node.expressions {
      last = Some(self.exec_expression(expression));
    }
    last.unwrap()
  }
}
