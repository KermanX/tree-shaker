use crate::ast::DeclarationKind;
use crate::entity::entity::Entity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::BindingRestElement;

impl<'a> Analyzer<'a> {
  pub fn declare_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    exporting: bool,
    kind: DeclarationKind,
  ) {
    self.declare_binding_pattern(&node.argument, exporting, kind);
  }

  pub fn init_binding_rest_element(&mut self, node: &'a BindingRestElement<'a>, init: Entity<'a>) {
    self.exec_binding_pattern(&node.argument, init);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_binding_rest_element(
    &self,
    node: &'a BindingRestElement<'a>,
    need_binding: bool,
  ) -> Option<BindingRestElement<'a>> {
    let BindingRestElement { span, argument, .. } = node;

    let argument = self.transform_binding_pattern(argument, need_binding);

    argument.map(|argument| self.ast_builder.binding_rest_element(*span, argument))
  }
}
