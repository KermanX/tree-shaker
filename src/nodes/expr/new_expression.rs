use crate::{
  analyzer::Analyzer,
  entity::{entity::Entity, unknown::UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, NewExpression, TSTypeParameterInstantiation};

impl<'a> Analyzer<'a> {
  pub fn exec_new_expression(&mut self, node: &'a NewExpression<'a>) -> Entity<'a> {
    let callee = self.exec_expression(&node.callee);
    let arguments = self.exec_arguments(&node.arguments);

    callee.consume(self);
    arguments.consume(self);

    UnknownEntity::new_object()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_new_expression(
    &self,
    node: &'a NewExpression<'a>,
    _need_val: bool,
  ) -> Option<Expression<'a>> {
    let NewExpression { span, callee, arguments, .. } = node;

    let callee = self.transform_expression(callee, true);
    let arguments = self.transform_arguments_need_call(arguments);

    Some(self.ast_builder.expression_new(
      *span,
      callee.unwrap(),
      arguments,
      None::<TSTypeParameterInstantiation>,
    ))
  }
}
