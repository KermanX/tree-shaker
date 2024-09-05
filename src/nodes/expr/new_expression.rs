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

    callee.consume_as_unknown(self);
    arguments.consume_as_unknown(self);

    UnknownEntity::new_unknown()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_new_expression(
    &self,
    node: &'a NewExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let NewExpression { span, callee, arguments, .. } = node;

    let callee = self.transform_expression(callee, true);

    Some(self.ast_builder.expression_new(
      *span,
      callee.unwrap(),
      self.clone_node(arguments),
      None::<TSTypeParameterInstantiation>,
    ))
  }
}
