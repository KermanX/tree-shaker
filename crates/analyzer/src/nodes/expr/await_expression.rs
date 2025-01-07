use crate::{
  analyzer::Analyzer,
  host::{EntityHost, Host},
};
use oxc::ast::ast::AwaitExpression;

#[allow(unused_variables)]
pub trait TraverseAwaitExpression<'a>: EntityHost<'a> {
  fn before_await_expression(&self, node: &'a AwaitExpression<'a>) {}
  fn after_await_expression(
    &self,
    node: &'a AwaitExpression<'a>,
    value: Self::Entity,
  ) -> Self::Entity {
    value
  }
}

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_await_expression(&mut self, node: &'a AwaitExpression<'a>) -> H::Entity {
    let call_scope = self.call_scope_mut();
    if !call_scope.is_async {
      self.add_diagnostic("SyntaxError: await is only valid in async functions");
    }

    self.host.before_await_expression(node);

    let value = self.exec_expression(&node.argument);
    let awaited = self.host.awaited(value);

    self.host.after_await_expression(node, awaited)
  }
}
