use crate::{
  ast::{Arguments, AstType2},
  symbol::arguments::{ArgumentsSource, ArgumentsSourceFromNode},
  transformer::Transformer,
  Analyzer,
};
use oxc::{
  ast::ast::{Argument, Expression},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::Arguments;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_arguments(
    &mut self,
    node: &'a Arguments<'a>,
  ) -> (bool, &'a dyn ArgumentsSource<'a>) {
    let mut effect = false;
    for argument in node {
      let expression = match argument {
        Argument::SpreadElement(node) => &node.argument,
        node => node.to_expression(),
      };
      effect |= self.exec_expression(expression).0;
    }
    (effect, self.allocator.alloc(ArgumentsSourceFromNode { node }))
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
