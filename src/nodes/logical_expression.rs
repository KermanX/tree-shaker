use crate::{analyzer::Analyzer, build_effect, entity::Entity, Transformer};
use oxc::{
  ast::ast::{Expression, LogicalExpression, LogicalOperator},
  span::GetSpan,
};
use std::rc::Rc;

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_left: bool,
  need_right: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_logical_expression(&mut self, node: &'a LogicalExpression) -> Entity {
    let data = self.load_data::<Data>(node);

    let left_val = self.exec_expression(&node.left);
    let right_val = self.exec_expression(&node.right);

    let (value, need_left, need_right) = match &node.operator {
      LogicalOperator::And => match left_val.to_boolean() {
        Entity::BooleanLiteral(true) => (right_val, false, true),
        Entity::BooleanLiteral(false) => (left_val, true, false),
        Entity::Union(_) => {
          (Entity::Union(vec![Rc::new(left_val), Rc::new(right_val)]), true, true)
        }
        _ => unreachable!(),
      },
      LogicalOperator::Or => match left_val.to_boolean() {
        Entity::BooleanLiteral(true) => (left_val, true, false),
        Entity::BooleanLiteral(false) => (right_val, false, true),
        Entity::Union(_) => {
          (Entity::Union(vec![Rc::new(left_val), Rc::new(right_val)]), true, true)
        }
        _ => unreachable!(),
      },
      _ => todo!(),
    };

    data.need_left = need_left;
    data.need_right = need_right;

    value
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_logical_expression(
    &self,
    node: LogicalExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(&node);
    let span = node.span();

    if need_val {
      let left = self.transform_expression(node.left, data.need_left);
      let right = self.transform_expression(node.right, data.need_right);
      match (data.need_left, data.need_right) {
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
        (false, true) => build_effect!(self.ast_builder, span, left, right),
        (false, false) => unreachable!(),
      }
    } else {
      let left = self.transform_expression(node.left, false);
      let right = self.transform_expression(node.right, false);
      build_effect!(self.ast_builder, span, left, right)
    }
  }
}
