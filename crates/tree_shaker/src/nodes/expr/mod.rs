use crate::{
  analyzer::Analyzer,
  entity::{Entity, LiteralCollector},
  utils::ast::AstKind2,
};
use ecma_analyzer::{EcmaAnalyzer, ExpressionAnalyzer};
use oxc::ast::ast::Expression;

mod array_expression;
mod literals;

#[derive(Debug, Default)]
struct Data<'a> {
  collector: LiteralCollector<'a>,
}

impl<'a> ExpressionAnalyzer<'a> for Analyzer<'a> {
  fn before_expression(&mut self, node: &'a Expression<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.push_span(node);
  }

  fn after_expression(&mut self, node: &'a Expression<'a>, value: Entity<'a>) -> Entity<'a>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.pop_span();
    let data = self.load_data::<Data>(AstKind2::Expression(node));
    data.collector.collect(self, value)
  }
}
