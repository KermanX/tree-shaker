use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::Declaration;

impl<'a> Analyzer<'a> {
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

impl<'a> Transformer<'a> {
  pub fn transform_declaration(&self, node: &'a Declaration<'a>) -> Option<Declaration<'a>> {
    match node {
      Declaration::VariableDeclaration(node) => self
        .transform_variable_declaration(node)
        .map(|decl| self.ast_builder.declaration_from_variable(decl)),
      Declaration::FunctionDeclaration(node) => {
        self.transform_function(node, false).map(|f| self.ast_builder.declaration_from_function(f))
      }
      Declaration::ClassDeclaration(node) => {
        self.transform_class(node, false).map(|c| self.ast_builder.declaration_from_class(c))
      }
      _ => unreachable!(),
    }
  }
}
