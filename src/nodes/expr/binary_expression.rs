use std::vec;

use crate::{
  analyzer::Analyzer,
  build_effect,
  entity::{
    entity::Entity,
    unknown::{UnknownEntity, UnknownEntityKind},
    utils::boolean_from_test_result,
  },
  transformer::Transformer,
};
use oxc::ast::ast::{BinaryExpression, BinaryOperator, Expression};

impl<'a> Analyzer<'a> {
  pub fn exec_binary_expression(&mut self, node: &'a BinaryExpression<'a>) -> Entity<'a> {
    let lhs = self.exec_expression(&node.left);
    let rhs = self.exec_expression(&node.right);

    let to_result =
      |result: Option<bool>| boolean_from_test_result(result, || vec![lhs.clone(), rhs.clone()]);

    match &node.operator {
      BinaryOperator::Equality => to_result(self.entity_op.eq(&lhs, &rhs)),
      BinaryOperator::Inequality => to_result(self.entity_op.neq(&lhs, &rhs)),
      BinaryOperator::StrictEquality => to_result(self.entity_op.strict_eq(&lhs, &rhs)),
      BinaryOperator::StrictInequality => to_result(self.entity_op.strict_neq(&lhs, &rhs)),
      BinaryOperator::LessThan => to_result(self.entity_op.lt(&lhs, &rhs, false)),
      BinaryOperator::LessEqualThan => to_result(self.entity_op.lt(&lhs, &rhs, true)),
      BinaryOperator::GreaterThan => to_result(self.entity_op.gt(&lhs, &rhs, false)),
      BinaryOperator::GreaterEqualThan => to_result(self.entity_op.gt(&lhs, &rhs, true)),
      BinaryOperator::Addition => self.entity_op.add(&lhs, &rhs),
      BinaryOperator::Subtraction
      | BinaryOperator::ShiftLeft
      | BinaryOperator::ShiftRight
      | BinaryOperator::ShiftRightZeroFill
      | BinaryOperator::Multiplication
      | BinaryOperator::Division
      | BinaryOperator::Remainder
      | BinaryOperator::BitwiseOR
      | BinaryOperator::BitwiseXOR
      | BinaryOperator::BitwiseAnd
      | BinaryOperator::Exponential => {
        // Can be number or bigint
        UnknownEntity::new_unknown_with_deps(vec![lhs, rhs])
      }
      BinaryOperator::In | BinaryOperator::Instanceof => {
        UnknownEntity::new_with_deps(UnknownEntityKind::Boolean, vec![lhs, rhs])
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_binary_expression(
    &self,
    node: &'a BinaryExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let BinaryExpression { span, operator, left, right } = node;

    let left = self.transform_expression(left, need_val);
    let right = self.transform_expression(right, need_val);

    if need_val {
      Some(self.ast_builder.expression_binary(*span, left.unwrap(), *operator, right.unwrap()))
    } else {
      build_effect!(self.ast_builder, *span, left, right)
    }
  }
}
