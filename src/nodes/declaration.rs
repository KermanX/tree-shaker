use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::Declaration;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_declaration(&mut self, node: &'a Declaration) -> bool {
    match node {
      Declaration::VariableDeclaration(node) => {
        let mut init_effect = false;
        for declarator in &node.declarations {
          init_effect |= self.exec_variable_declarator(declarator);
        }
        init_effect
      }
      Declaration::FunctionDeclaration(node) => self.exec_function(node).0,
      _ => todo!(),
    }
  }
}

impl<'a> Transformer<'a> {}
