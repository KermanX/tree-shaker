use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::Declaration;

impl<'a> Analyzer<'a> {
  pub fn exec_declaration(&mut self, node: &'a Declaration<'a>, exporting: bool) {
    match node {
      Declaration::VariableDeclaration(node) => {
        self.exec_variable_declaration(node, exporting, None);
      }
      Declaration::FunctionDeclaration(node) => {
        self.exec_function(node, exporting);
      }
      Declaration::ClassDeclaration(node) => todo!(),
      Declaration::UsingDeclaration(node) => todo!(),
      _ => unreachable!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_declaration(&mut self, node: Declaration<'a>) -> Option<Declaration<'a>> {
    match node {
      Declaration::VariableDeclaration(node) => self
        .transform_variable_declaration(node.unbox())
        .map(|decl| self.ast_builder.declaration_from_variable(decl)),
      Declaration::FunctionDeclaration(node) => self
        .transform_function(node.unbox(), false)
        .map(|f| self.ast_builder.declaration_from_function(f)),
      _ => todo!(),
    }
  }
}
