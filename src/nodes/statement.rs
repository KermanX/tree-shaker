use crate::TreeShaker;
use oxc::ast::{ast::Statement, match_declaration};

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_statement(&mut self, node: &'a Statement) {
    let data = self.load_data::<Data>(node);

    match node {
      match_declaration!(Statement) => {
        let node = node.to_declaration();
        self.exec_declaration(node);
      }
      _ => todo!(),
    }
  }

  pub(crate) fn transform_statement(&mut self, node: Statement<'a>) -> Option<Statement<'a>> {
    match node {
      _ => todo!(),
    }
  }
}
