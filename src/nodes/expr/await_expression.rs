use crate::{analyzer::Analyzer, entity::Entity, transformer::Transformer};
use oxc::ast::ast::{AwaitExpression, Expression};

impl<'a> Analyzer<'a> {
  pub fn exec_await_expression(&mut self, node: &'a AwaitExpression<'a>) -> Entity<'a> {
    let value = self.exec_expression(&node.argument);
    let awaited = value.r#await(self);

    let call_scope = self.call_scope_mut();
    if !call_scope.is_async {
      // TODO: throw warning
    }

    awaited
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
