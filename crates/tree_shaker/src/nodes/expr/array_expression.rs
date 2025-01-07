use crate::TreeShaker;
use ecma_analyzer::{Analyzer, ArrayExpressionAnalyzer};
use oxc::ast::ast::{ArrayExpression, ArrayExpressionElement, SpreadElement};

pub struct Context {}

impl<'a> ArrayExpressionAnalyzer<'a> for TreeShaker<'a> {
  type Context = Context;
  fn before_array_expression(&mut self, node: &'a ArrayExpression<'a>) -> Self::Context {
    Context {}
  }

  fn init_element(
    &mut self,
    node: &'a ArrayExpressionElement<'a>,
    context: &mut Self::Context,
    value: <TreeShaker<'a> as Analyzer<'a>>::Entity,
  ) {
  }

  fn init_spread(
    &mut self,
    node: &'a SpreadElement<'a>,
    context: &mut Self::Context,
    value: <TreeShaker<'a> as Analyzer<'a>>::Entity,
  ) {
  }

  fn after_array_expression(
    &mut self,
    context: Self::Context,
  ) -> <TreeShaker<'a> as Analyzer<'a>>::Entity {
    todo!()
  }
}
