use crate::Analyzer;
use oxc::ast::ast::ExportNamedDeclaration;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_export_named_declaration(&mut self, node: &'a ExportNamedDeclaration) {
    node.declaration.as_ref().map(|declaration| self.exec_declaration(declaration));
  }
}
