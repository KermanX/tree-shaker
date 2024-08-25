mod block_statement;
mod declaration;
mod if_statement;
mod module_declaration;
mod return_statement;
mod while_statement;

use crate::{transformer::Transformer, Analyzer};
use oxc::{
  allocator::Vec,
  ast::{
    ast::{ExpressionStatement, Statement},
    match_declaration, match_module_declaration,
  },
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_statement(&mut self, node: &'a Statement) {
    match node {
      match_declaration!(Statement) => {
        let node = node.to_declaration();
        self.exec_declaration(node, false);
      }
      match_module_declaration!(Statement) => {
        let node = node.to_module_declaration();
        self.exec_module_declaration(node);
      }
      Statement::ExpressionStatement(node) => {
        self.exec_expression(&node.expression);
      }
      Statement::BlockStatement(node) => self.exec_block_statement(node),
      Statement::IfStatement(node) => self.exec_if_statement(node),
      Statement::WhileStatement(node) => self.exec_while_statement(node),
      Statement::ReturnStatement(node) => self.exec_return_statement(node),
      _ => todo!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_statement(&mut self, node: Statement<'a>) -> Option<Statement<'a>> {
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
      Statement::BlockStatement(node) => self.transform_block_statement(node.unbox()),
      Statement::IfStatement(node) => self.transform_if_statement(node.unbox()),
      Statement::WhileStatement(node) => self.transform_while_statement(node.unbox()),
      Statement::ReturnStatement(node) => self.transform_return_statement(node.unbox()),
      _ => todo!(),
    }
  }

  pub(crate) fn transform_statements(
    &mut self,
    nodes: Vec<'a, Statement<'a>>,
  ) -> Vec<'a, Statement<'a>> {
    let mut result = self.ast_builder.vec::<Statement>();
    for node in nodes {
      let new_statement = self.transform_statement(node);
      if let Some(transformed) = new_statement {
        result.push(transformed);
      }
    }
    result
  }
}
