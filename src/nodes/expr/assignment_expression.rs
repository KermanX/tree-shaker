use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::ast::ast::{AssignmentExpression, Expression};

impl<'a> Analyzer<'a> {
  pub fn exec_assignment_expression(&mut self, node: &'a AssignmentExpression<'a>) -> Entity<'a> {
    let value = self.exec_expression(&node.right);
    self.exec_assignment_target(&node.left, value.clone());
    value
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_assignment_expression(
    &self,
    node: &'a AssignmentExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let AssignmentExpression { span, operator, left, right } = node;

    let (left_is_empty, left) = self.transform_assignment_target(left, false, false);
    let right = self.transform_expression(right, need_val || !left_is_empty);

    match (left, right) {
      (Some(left), Some(right)) => {
        Some(self.ast_builder.expression_assignment(*span, *operator, left, right))
      }
      (None, Some(right)) => Some(right),
      (None, None) => None,
      _ => unreachable!(),
    }
  }
}
