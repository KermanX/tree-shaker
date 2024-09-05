use crate::{
  analyzer::Analyzer,
  build_effect,
  entity::{entity::Entity, utils::boolean_from_test_result},
  transformer::Transformer,
};
use oxc::ast::ast::{BinaryExpression, BinaryOperator, Expression};

impl<'a> Analyzer<'a> {
  pub fn exec_binary_expression(&mut self, node: &'a BinaryExpression<'a>) -> Entity<'a> {
    let lhs = self.exec_expression(&node.left);
    let rhs = self.exec_expression(&node.right);

    match &node.operator {
      BinaryOperator::Equality => todo!(),
      BinaryOperator::Inequality => todo!(),
      BinaryOperator::StrictEquality => {
        boolean_from_test_result(self.entity_op.strict_eq(&lhs, &rhs), || vec![lhs, rhs])
      }
      BinaryOperator::StrictInequality => {
        boolean_from_test_result(self.entity_op.strict_eq(&lhs, &rhs).map(|v| !v), || {
          vec![lhs, rhs]
        })
      }
      BinaryOperator::LessThan => todo!(),
      BinaryOperator::LessEqualThan => todo!(),
      BinaryOperator::GreaterThan => todo!(),
      BinaryOperator::GreaterEqualThan => todo!(),
      BinaryOperator::ShiftLeft => todo!(),
      BinaryOperator::ShiftRight => todo!(),
      BinaryOperator::ShiftRightZeroFill => todo!(),
      BinaryOperator::Addition => self.entity_op.add(&lhs, &rhs),
      BinaryOperator::Subtraction => todo!(),
      BinaryOperator::Multiplication => todo!(),
      BinaryOperator::Division => todo!(),
      BinaryOperator::Remainder => todo!(),
      BinaryOperator::BitwiseOR => todo!(),
      BinaryOperator::BitwiseXOR => todo!(),
      BinaryOperator::BitwiseAnd => todo!(),
      BinaryOperator::In => todo!(),
      BinaryOperator::Instanceof => todo!(),
      BinaryOperator::Exponential => todo!(),
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
