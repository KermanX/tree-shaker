use crate::TreeShakerImpl;
use oxc::ast::{ast::Statement, match_declaration};

#[derive(Debug, Default, Clone)]
pub struct Data {
  included: bool,
}

impl<'a> TreeShakerImpl<'a> {
  pub(crate) fn exec_statement(&mut self, node: &'a Statement) {
    let data = self.load_data::<Data>(node);
    data.included = true;

    match node {
      match_declaration!(Statement) => {
        let node = node.to_declaration();
        self.exec_declaration(node, None);
      }
      _ => todo!(),
    }
  }
}
