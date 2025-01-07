use crate::{host::Host, analyzer::Analyzer, ast::DeclarationKind};
use oxc::{
  allocator,
  ast::ast::{VariableDeclaration, VariableDeclarationKind},
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn declare_variable_declaration(
    &mut self,
    node: &'a VariableDeclaration<'a>,
    exporting: bool,
  ) {
    let kind = match &node.kind {
      VariableDeclarationKind::Var => DeclarationKind::Var,
      VariableDeclarationKind::Let => DeclarationKind::Let,
      VariableDeclarationKind::Const => DeclarationKind::Const,
      _ => unimplemented!("using statement"),
    };

    for declarator in &node.declarations {
      self.declare_variable_declarator(declarator, exporting, kind);
    }
  }

  pub fn init_variable_declaration(
    &mut self,
    node: &'a VariableDeclaration<'a>,
    init: Option<H::Entity>,
  ) {
    if init.is_some() {
      assert_eq!(node.declarations.len(), 1);
    }

    for declarator in &node.declarations {
      self.init_variable_declarator(declarator, init);
    }
  }
}

