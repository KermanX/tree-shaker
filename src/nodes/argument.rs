use crate::{symbol::SymbolSource, transformer::Transformer, Analyzer};
use oxc::{
  ast::ast::{Argument, Expression},
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_argument(&mut self, node: &'a Argument) -> (bool, (bool, SymbolSource<'a>)) {
    let (expended, node) = match node {
      Argument::SpreadElement(node) => (true, &node.argument),
      _ => (false, node.to_expression()),
    };
    let effect = self.exec_expression(node).0;
    (effect, (expended, SymbolSource::Expression(node)))
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_argument_need_val(&self, node: Argument<'a>) -> Argument<'a> {
    let span = node.span();
    match node {
      Argument::SpreadElement(node) => {
        let expr = self.transform_expression(node.unbox().argument, true).unwrap();
        self.ast_builder.argument_spread_element(span, expr)
      }
      _ => {
        let expr = self.transform_expression(node.try_into().unwrap(), true).unwrap();
        self.ast_builder.argument_expression(expr)
      }
    }
  }

  pub(crate) fn transform_argument_no_val(&self, node: Argument<'a>) -> Option<Expression<'a>> {
    let span = node.span();
    match node {
      Argument::SpreadElement(node) => self.transform_expression(node.unbox().argument, false),
      _ => self.transform_expression(node.try_into().unwrap(), false),
    }
  }
}
