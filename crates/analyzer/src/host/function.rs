use super::entity::EntityHost;
use oxc::ast::ast::{ArrowFunctionExpression, CallExpression, Function};

pub trait FunctionHost<'a>: EntityHost<'a> {
  fn new_arrow_function(&self, node: &'a ArrowFunctionExpression<'a>) -> Self::Entity;
  fn new_function_expression(&self, node: &'a Function<'a>) -> Self::Entity;

  fn call(
    &self,
    node: CallExpression<'a>,
    function: Self::Entity,
    this: Self::Entity,
    args: Self::Entity,
  ) -> Self::Entity;
}
