use crate::entity::entity::Entity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::BindingRestElement;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    init_val: Entity<'a>,
    exporting: bool,
  ) {
    self.exec_binding_pattern(&node.argument, (false, init_val), exporting)
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
