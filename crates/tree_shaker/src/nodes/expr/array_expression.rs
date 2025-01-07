use crate::TreeShaker;
use ecma_analyzer::{Analyzer, ArrayExpressionAnalyzer};
use oxc::ast::ast::ArrayExpression;

struct Context {}

impl<'a> ArrayExpressionAnalyzer<'a> for TreeShaker<'a> {
  type Context = Context;
  fn before_array_expression(&mut self, node: &'a ArrayExpression<'a>) -> Self::Context
  where
    Self: Analyzer<'a>,
  {
    Context {}
  }

  fn init_element(
    &mut self,
    node: &'a ArrayExpressionElement<'a>,
    context: &mut Self::Context,
    value: Self::Entity,
  ) where
    Self: Analyzer<'a>,
  {
  }

  fn init_spread(
    &mut self,
    node: &'a SpreadElement<'a>,
    context: &mut Self::Context,
    value: Self::Entity,
  ) where
    Self: Analyzer<'a>,
  {
  }

  fn after_array_expression(&mut self, context: Self::Context) -> Self::Entity
  where
    Self: Analyzer<'a>,
  {
    todo!()
  }
}
