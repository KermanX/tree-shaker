use crate::{transformer::Transformer, Analyzer};
use oxc::{
  ast::{
    ast::{ExpressionStatement, Statement},
    match_declaration,
  },
  span::GetSpan,
};

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_statement(&mut self, node: &'a Statement) {
    match node {
      match_declaration!(Statement) => {
        let node = node.to_declaration();
        self.exec_declaration(node);
      }
      Statement::ExpressionStatement(node) => {
        self.exec_expression(&node.expression);
      }
      _ => todo!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_statement(&self, node: Statement<'a>) -> Option<Statement<'a>> {
    let span = node.span();
    match node {
      Statement::ExpressionStatement(node) => {
        let ExpressionStatement { expression, .. } = node.unbox();
        self
          .transform_expression(expression, false)
          .map(|expr| self.ast_builder.statement_expression(span, expr))
      }
      _ => todo!(),
    }
  }
}
