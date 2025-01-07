use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::ast::Declaration;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn declare_declaration(&mut self, node: &'a Declaration<'a>, exporting: bool) {
    match node {
      Declaration::VariableDeclaration(node) => {
        self.declare_variable_declaration(node, exporting);
      }
      Declaration::FunctionDeclaration(node) => {
        self.declare_function(node, exporting);
      }
      Declaration::ClassDeclaration(node) => {
        self.declare_class(node, exporting);
      }
      _ => unreachable!(),
    }
  }

  pub fn init_declaration(&mut self, node: &'a Declaration<'a>) {
    match node {
      Declaration::VariableDeclaration(node) => {
        self.init_variable_declaration(node, None);
      }
      Declaration::FunctionDeclaration(_node) => {
        // Nothing to do
      }
      Declaration::ClassDeclaration(node) => {
        self.init_class(node);
      }
      _ => unreachable!(),
    }
  }
}

