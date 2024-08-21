use crate::{transformer::Transformer, Analyzer};
use oxc::ast::{ast::Statement, match_declaration};

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_statement(&mut self, node: &'a Statement) {
    match node {
      match_declaration!(Statement) => {
        let node = node.to_declaration();
        self.exec_declaration(node);
      }
      _ => todo!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_statement(&self, node: Statement<'a>) -> Option<Statement<'a>> {
    match node {
      _ => todo!(),
    }
  }
}
