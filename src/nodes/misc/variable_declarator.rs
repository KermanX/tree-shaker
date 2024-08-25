use crate::entity::literal::LiteralEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::{ast::ast::VariableDeclarator, span::GetSpan};
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_variable_declarator(&mut self, node: &'a VariableDeclarator) {
    let init = match &node.init {
      Some(init) => self.exec_expression(init),
      None => Rc::new(LiteralEntity::Undefined),
    };

    self.exec_binding_pattern(&node.id, init);
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_variable_declarator(
    &self,
    node: VariableDeclarator<'a>,
  ) -> Option<VariableDeclarator<'a>> {
    let VariableDeclarator { span, kind, id, init, .. } = node;

    let id_span = id.span();
    let id = self.transform_binding_pattern(id);

    let init = init.and_then(|init| self.transform_expression(init, id.is_some()));

    match (id, init) {
      (None, None) => None,
      (id, init) => Some(self.ast_builder.variable_declarator(
        span,
        kind,
        id.unwrap_or_else(|| self.build_unused_binding_pattern(id_span)),
        init,
        false,
      )),
    }
  }
}
