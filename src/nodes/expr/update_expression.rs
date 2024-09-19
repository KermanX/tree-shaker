use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::{
  ast::ast::{BinaryOperator, Expression, NumberBase, UpdateExpression, UpdateOperator},
  span::SPAN,
};

impl<'a> Analyzer<'a> {
  pub fn exec_update_expression(&mut self, node: &'a UpdateExpression<'a>) -> Entity<'a> {
    let (value, cache) = self.exec_simple_assignment_target_read(&node.argument);
    let updated_value = self.entity_op.update(&value, &node.operator);
    self.exec_simple_assignment_target_write(&node.argument, updated_value.clone(), cache);
    if node.prefix {
      updated_value
    } else {
      value
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_update_expression(
    &self,
    node: &'a UpdateExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let UpdateExpression { span, argument, operator, prefix, .. } = node;

    let argument_write = self.transform_simple_assignment_target_write(argument);

    if let Some(argument_write) = argument_write {
      Some(self.ast_builder.expression_update(*span, *operator, *prefix, argument_write))
    } else {
      if need_val {
        let argument = self.transform_simple_assignment_target_read(argument, true).unwrap();
        let operator = match operator {
          UpdateOperator::Increment => BinaryOperator::Addition,
          UpdateOperator::Decrement => BinaryOperator::Subtraction,
        };
        let rhs = self.ast_builder.expression_numeric_literal(SPAN, 1f64, "1", NumberBase::Decimal);
        Some(self.ast_builder.expression_binary(*span, argument, operator, rhs))
      } else {
        self.transform_simple_assignment_target_read(argument, false)
      }
    }
  }
}
