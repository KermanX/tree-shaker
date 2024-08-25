use crate::ast::AstType2;
use crate::entity::entity::Entity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::BindingRestElement;

const AST_TYPE: AstType2 = AstType2::BindingRestElement;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    init_val: Entity<'a>,
  ) {
    self.exec_binding_pattern(&node.argument, init_val)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_binding_rest_element(
    &mut self,
    node: BindingRestElement<'a>,
  ) -> Option<BindingRestElement<'a>> {
    let BindingRestElement { span, argument, .. } = node;

    self
      .transform_binding_pattern(argument)
      .map(|argument| self.ast_builder.binding_rest_element(span, argument))
  }
}
