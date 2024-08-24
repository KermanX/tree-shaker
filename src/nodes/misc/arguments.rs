use crate::{
  ast::{Arguments, AstType2},
  entity::{arguments::ArgumentsEntity, entity::Entity},
  transformer::Transformer,
  Analyzer,
};
use oxc::{
  ast::ast::{Argument, Expression},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::Arguments;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_arguments(&mut self, node: &'a Arguments<'a>) -> Entity<'a> {
    let mut arguments = vec![];
    for argument in node {
      arguments.push(match argument {
        Argument::SpreadElement(node) => (true, self.exec_expression(&node.argument)),
        node => (false, self.exec_expression(node.to_expression())),
      })
    }
    ArgumentsEntity::new(arguments)
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
