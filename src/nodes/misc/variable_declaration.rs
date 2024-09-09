use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::ast::ast::VariableDeclaration;

impl<'a> Analyzer<'a> {
  pub fn exec_variable_declaration(
    &mut self,
    node: &'a VariableDeclaration<'a>,
    exporting: bool,
    init: Option<Entity<'a>>,
  ) {
    for declarator in &node.declarations {
      self.exec_variable_declarator(declarator, init.clone(), exporting, node.kind);
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_variable_declaration(
    &self,
    node: &'a VariableDeclaration<'a>,
  ) -> Option<VariableDeclaration<'a>> {
    let VariableDeclaration { span, kind, declarations, .. } = node;
    let mut transformed_decls = self.ast_builder.vec();
    for declarator in declarations {
      let declarator = self.transform_variable_declarator(declarator);
      if let Some(declarator) = declarator {
        transformed_decls.push(declarator);
      }
    }
    if transformed_decls.is_empty() {
      None
    } else {
      Some(self.ast_builder.variable_declaration(*span, *kind, transformed_decls, false))
    }
  }
}
