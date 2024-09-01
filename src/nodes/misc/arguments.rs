use crate::{
  ast::Arguments,
  entity::{arguments::ArgumentsEntity, entity::Entity},
  transformer::Transformer,
  Analyzer,
};
use oxc::{
  ast::ast::{Argument, Expression},
  span::GetSpan,
};

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
  pub(crate) fn transform_arguments_need_call(&mut self, node: Arguments<'a>) -> Arguments<'a> {
    let mut arguments = self.ast_builder.vec();
    for argument in node {
      arguments.push(self.transform_argument_need_call(argument));
    }
    arguments
  }

  fn transform_argument_need_call(&mut self, node: Argument<'a>) -> Argument<'a> {
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

  pub(crate) fn transform_arguments_no_call(
    &mut self,
    node: Arguments<'a>,
  ) -> Vec<Option<Expression<'a>>> {
    node.into_iter().map(|arg| self.transform_argument_no_call(arg)).collect()
  }

  fn transform_argument_no_call(&mut self, node: Argument<'a>) -> Option<Expression<'a>> {
    match node {
      Argument::SpreadElement(node) => self.transform_expression(node.unbox().argument, false),
      _ => self.transform_expression(node.try_into().unwrap(), false),
    }
  }
}
