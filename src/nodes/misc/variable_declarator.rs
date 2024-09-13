use crate::ast::DeclarationKind;
use crate::entity::entity::Entity;
use crate::entity::literal::LiteralEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::{ast::ast::VariableDeclarator, span::GetSpan};

impl<'a> Analyzer<'a> {
  pub fn declare_variable_declarator(
    &mut self,
    node: &'a VariableDeclarator,
    exporting: bool,
    kind: DeclarationKind,
  ) {
    self.declare_binding_pattern(&node.id, exporting, kind);
  }

  pub fn exec_variable_declarator(
    &mut self,
    node: &'a VariableDeclarator,
    init: Option<Entity<'a>>,
  ) {
    let init = init.unwrap_or_else(|| match &node.init {
      Some(init) => self.exec_expression(init),
      None => LiteralEntity::new_undefined(),
    });

    self.exec_binding_pattern(&node.id, (false, init));
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_variable_declarator(
    &self,
    node: &'a VariableDeclarator<'a>,
  ) -> Option<VariableDeclarator<'a>> {
    let VariableDeclarator { span, kind, id, init, .. } = node;

    let id_span = id.span();
    let id = self.transform_binding_pattern(id);

    let init = init.as_ref().and_then(|init| self.transform_expression(init, id.is_some()));

    match (id, init) {
      (None, None) => None,
      (id, init) => Some(self.ast_builder.variable_declarator(
        *span,
        *kind,
        id.unwrap_or_else(|| self.build_unused_binding_pattern(id_span)),
        init,
        false,
      )),
    }
  }
}
