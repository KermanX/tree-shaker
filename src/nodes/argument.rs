use crate::{entity::Entity, symbol::SymbolSource, transformer::Transformer, Analyzer};
use oxc::{
  ast::ast::{Argument, Expression},
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_argument(&mut self, node: &'a Argument) -> (bool, SymbolSource<'a>) {
    match node {
      Argument::SpreadElement(node) => (true, SymbolSource::Expression(&node.argument)),
      _ => {
        let node = node.to_expression();
        (false, SymbolSource::Expression(node))
      }
    }
  }

  /// returns (expended, value)
  pub(crate) fn calc_argument(&self, node: &'a Argument) -> (bool, Entity) {
    match node {
      Argument::SpreadElement(node) => (true, self.calc_expression(&node.argument)),
      _ => {
        let node = node.to_expression();
        (false, self.calc_expression(node))
      }
    }
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
