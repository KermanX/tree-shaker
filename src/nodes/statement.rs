use crate::{transformer::Transformer, Analyzer};
use oxc::{
  ast::{
    ast::{ExpressionStatement, Statement},
    match_declaration, match_module_declaration,
  },
  span::GetSpan,
  syntax::node,
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
      match_module_declaration!(Statement) => {
        let node = node.to_module_declaration();
        self.exec_module_declaration(node);
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
      match_declaration!(Statement) => self
        .transform_declaration(node.try_into().unwrap())
        .map(|decl| self.ast_builder.statement_declaration(decl)),
      match_module_declaration!(Statement) => {
        Some(self.ast_builder.statement_module_declaration(
          self.transform_module_declaration(node.try_into().unwrap()),
        ))
      }
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
