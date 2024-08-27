use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::ast::ast::{Expression, ParenthesizedExpression};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_parenthesized_expression(
    &mut self,
    node: &'a ParenthesizedExpression<'a>,
  ) -> Entity<'a> {
    self.exec_expression(&node.expression)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_parenthesized_expression(
    &mut self,
    node: ParenthesizedExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let ParenthesizedExpression { span, expression } = node;
    self
      .transform_expression(expression, need_val)
      .map(|expression| self.ast_builder.expression_parenthesized(span, expression))
  }
}