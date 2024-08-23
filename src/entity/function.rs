use super::EntityValue;
use crate::{symbol::arguments::ArgumentsSource, Analyzer};
use oxc::span::Span;

#[derive(Debug, Default, Clone)]
pub struct FunctionEntity {
  span: Span,
}

impl FunctionEntity {
  pub fn new(span: Span) -> Self {
    FunctionEntity { span }
  }

  pub(crate) fn call<'a>(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: EntityValue,
    args: &'a dyn ArgumentsSource<'a>,
  ) -> (bool, EntityValue) {
    let node = analyzer.functions.get(&self.span).unwrap();
    analyzer.call_function(node, this, args)
  }
}
