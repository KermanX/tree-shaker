use oxc::ast::ast::{BinaryExpression, BinaryOperator, Expression};

use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_binary_expression(&mut self, node: &'a BinaryExpression<'a>) -> Entity<'a> {
    let left = self.exec_expression(&node.left);
    let right = self.exec_expression(&node.right);

    match &node.operator {
      BinaryOperator::Equality => todo!(),
      BinaryOperator::Inequality => todo!(),
      BinaryOperator::StrictEquality => todo!(),
      BinaryOperator::StrictInequality => todo!(),
      BinaryOperator::LessThan => todo!(),
      BinaryOperator::LessEqualThan => todo!(),
      BinaryOperator::GreaterThan => todo!(),
      BinaryOperator::GreaterEqualThan => todo!(),
      BinaryOperator::ShiftLeft => todo!(),
      BinaryOperator::ShiftRight => todo!(),
      BinaryOperator::ShiftRightZeroFill => todo!(),
      BinaryOperator::Addition => todo!(),
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
  pub(crate) fn transform_binary_expression(
    &self,
    node: BinaryExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let BinaryExpression { span, operator, left, right } = node;

    match operator {
      BinaryOperator::Equality => todo!(),
      BinaryOperator::Inequality => todo!(),
      BinaryOperator::StrictEquality => todo!(),
      BinaryOperator::StrictInequality => todo!(),
      BinaryOperator::LessThan => todo!(),
      BinaryOperator::LessEqualThan => todo!(),
      BinaryOperator::GreaterThan => todo!(),
      BinaryOperator::GreaterEqualThan => todo!(),
      BinaryOperator::ShiftLeft => todo!(),
      BinaryOperator::ShiftRight => todo!(),
      BinaryOperator::ShiftRightZeroFill => todo!(),
      BinaryOperator::Addition => todo!(),
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
