use crate::entity::entity::Entity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::{BindingRestElement, VariableDeclarationKind};

impl<'a> Analyzer<'a> {
  pub fn exec_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    effect_and_init: (bool, Entity<'a>),
    exporting: bool,
    kind: VariableDeclarationKind,
  ) {
    self.exec_binding_pattern(&node.argument, effect_and_init, exporting, kind);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_binding_rest_element(
    &self,
    node: &'a BindingRestElement<'a>,
  ) -> Option<BindingRestElement<'a>> {
    let BindingRestElement { span, argument, .. } = node;

    let argument = self.transform_binding_pattern(argument);

    argument.map(|argument| self.ast_builder.binding_rest_element(*span, argument))
  }
}
