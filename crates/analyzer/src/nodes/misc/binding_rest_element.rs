use crate::{host::Host, analyzer::Analyzer, ast::DeclarationKind};
use oxc::ast::ast::BindingRestElement;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn declare_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    exporting: bool,
    kind: DeclarationKind,
  ) {
    self.declare_binding_pattern(&node.argument, exporting, kind);
  }

  pub fn init_binding_rest_element(&mut self, node: &'a BindingRestElement<'a>, init: H::Entity) {
    self.init_binding_pattern(&node.argument, Some(init));
  }
}

