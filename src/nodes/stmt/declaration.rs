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
      Declaration::ClassDeclaration(node) => {
        self.exec_class(node, exporting);
      }
      Declaration::UsingDeclaration(node) => todo!(),
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
      Declaration::UsingDeclaration(node) => todo!(),
      _ => unreachable!(),
    }
  }
}
