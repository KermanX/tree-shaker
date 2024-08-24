use crate::ast::AstType2;
use crate::entity::entity::Entity;
use crate::entity::union::UnionEntity;
use crate::{analyzer::Analyzer, build_effect, Transformer};
use oxc::{
  ast::ast::{Expression, LogicalExpression, LogicalOperator},
  span::GetSpan,
};
use std::rc::Rc;

const AST_TYPE: AstType2 = AstType2::LogicalExpression;

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_left: bool,
  need_right: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_logical_expression(&mut self, node: &'a LogicalExpression<'a>) -> Entity<'a> {
    let left = self.exec_expression(&node.left);
    let mut exec_right = || self.exec_expression(&node.right);
    let mut exec_unknown =
      || (Rc::new(UnionEntity(vec![left.clone(), exec_right()])) as Entity<'a>, true, true);

    let (value, need_left_val, need_right_val) = match &node.operator {
      LogicalOperator::And => match left.test_truthy() {
        Some(true) => (exec_right(), false, true),
        Some(false) => (left, true, false),
        None => exec_unknown(),
      },
      LogicalOperator::Or => match left.test_truthy() {
        Some(true) => (left, true, false),
        Some(false) => (exec_right(), false, true),
        None => exec_unknown(),
      },
      LogicalOperator::Coalesce => match left.test_nullish() {
        Some(true) => (exec_right(), false, true),
        Some(false) => (left, true, false),
        None => exec_unknown(),
      },
    };

    let data = self.load_data::<Data>(AST_TYPE, node);

    data.need_left |= need_left_val;
    data.need_right |= need_right_val;

    value
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_logical_expression(
    &self,
    node: LogicalExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);
    let span = node.span();

    let need_left = need_val && data.need_left;
    let need_right = need_val && data.need_right;

    let left = self.transform_expression(node.left, need_left);
    let right = self.transform_expression(node.right, need_right);
    match (need_left, need_right) {
      (true, true) => Some(self.ast_builder.expression_logical(
        span,
        left.unwrap(),
        node.operator,
        right.unwrap(),
      )),
      (true, false) => {
        if let Some(right) = right {
          Some(self.ast_builder.expression_logical(span, left.unwrap(), node.operator, right))
        } else {
          left
        }
      }
      (false, true) => Some(build_effect!(self.ast_builder, span, left; right.unwrap())),
      (false, false) => build_effect!(self.ast_builder, span, left),
    }
  }
}
