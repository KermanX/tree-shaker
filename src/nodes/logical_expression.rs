use crate::ast_type::AstType2;
use crate::{analyzer::Analyzer, build_effect, entity::Entity, Transformer};
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
  pub(crate) fn exec_logical_expression(&mut self, node: &'a LogicalExpression) -> (bool, Entity) {
    let (left_effect, left_val) = self.exec_expression(&node.left);
    let mut right_effect = false;
    let mut exec_right = || {
      let (effect, val) = self.exec_expression(&node.right);
      right_effect = effect;
      val
    };
    let mut exec_unknown =
      || (Entity::Union(vec![Rc::new(left_val.clone()), Rc::new(exec_right())]), true, true);

    let (value, need_left_val, need_right_val) = match &node.operator {
      LogicalOperator::And => match left_val.to_boolean() {
        Some(true) => (exec_right(), false, true),
        Some(false) => (left_val, true, false),
        None => exec_unknown(),
      },
      LogicalOperator::Or => match left_val.to_boolean() {
        Some(true) => (left_val, true, false),
        Some(false) => (exec_right(), false, true),
        None => exec_unknown(),
      },
      LogicalOperator::Coalesce => match left_val.is_null_or_undefined() {
        Some(true) => (exec_right(), false, true),
        Some(false) => (left_val, true, false),
        None => exec_unknown(),
      },
    };

    let data = self.load_data::<Data>(AST_TYPE, node);

    data.need_left |= left_effect || need_left_val;
    data.need_right |= need_right_val;

    (left_effect || right_effect, value)
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
      (false, true) => build_effect!(self.ast_builder, span, left; right.unwrap()),
      (false, false) => unreachable!(),
    }
  }
}
