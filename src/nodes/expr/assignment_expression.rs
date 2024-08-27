use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::ast::ast::{AssignmentExpression, Expression};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_assignment_expression(
    &mut self,
    node: &'a AssignmentExpression<'a>,
  ) -> Entity<'a> {
    let value = self.exec_expression(&node.right);
    self.exec_assignment_target(&node.left, value.clone());
    value
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_assignment_expression(
    &mut self,
    node: AssignmentExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let AssignmentExpression { span, left, operator, right } = node;

    let left = self.transform_assignment_target(left);
    let right = self.transform_expression(right, left.is_some());

    todo!()
  }
}
