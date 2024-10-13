use crate::{
  analyzer::Analyzer, consumable::box_consumable, entity::Entity, transformer::Transformer,
};
use oxc::ast::ast::{AwaitExpression, Expression};

impl<'a> Analyzer<'a> {
  pub fn exec_await_expression(&mut self, node: &'a AwaitExpression<'a>) -> Entity<'a> {
    let call_scope = self.call_scope_mut();
    if !call_scope.is_async {
      self.add_diagnostic("SyntaxError: await is only valid in async functions");
    }

    self.refer_to_global();

    let value = self.exec_expression(&node.argument);
    value.r#await(self, box_consumable(()))
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_await_expression(
    &self,
    node: &'a AwaitExpression<'a>,
    _need_val: bool,
  ) -> Option<Expression<'a>> {
    let AwaitExpression { span, argument, .. } = node;

    let argument = self.transform_expression(argument, true).unwrap();
    Some(self.ast_builder.expression_await(*span, argument))
  }
}
