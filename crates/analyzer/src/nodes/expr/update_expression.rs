use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  ast::ast::{
    BinaryOperator, Expression, NumberBase, UnaryOperator, UpdateExpression, UpdateOperator,
  },
  span::SPAN,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_update_expression(&mut self, node: &'a UpdateExpression<'a>) -> H::Entity {
    let (value, cache) = self.exec_simple_assignment_target_read(&node.argument);
    let numeric_value = value.get_to_numeric(self);
    let updated_value = self.entity_op.update(self, numeric_value, node.operator);
    self.exec_simple_assignment_target_write(&node.argument, updated_value, cache);
    if node.prefix {
      updated_value
    } else {
      numeric_value
    }
  }
}
