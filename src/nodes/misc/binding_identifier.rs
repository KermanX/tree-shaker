use crate::{
  analyzer::Analyzer,
  ast::{AstKind2, DeclarationKind},
  consumable::box_consumable,
  entity::Entity,
  transformer::Transformer,
};
use oxc::ast::ast::BindingIdentifier;

impl<'a> Analyzer<'a> {
  pub fn declare_binding_identifier(
    &mut self,
    node: &'a BindingIdentifier<'a>,
    exporting: bool,
    kind: DeclarationKind,
  ) {
    let symbol = node.symbol_id.get().unwrap();
    let dep = box_consumable(AstKind2::BindingIdentifier(node));
    self.declare_symbol(symbol, dep, exporting, kind, None);
  }

  pub fn init_binding_identifier(
    &mut self,
    node: &'a BindingIdentifier<'a>,
    init: Option<Entity<'a>>,
  ) {
    let symbol = node.symbol_id.get().unwrap();
    let dep = box_consumable(AstKind2::BindingIdentifier(node));
    self.init_symbol(symbol, init, dep);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_binding_identifier(
    &self,
    node: &'a BindingIdentifier<'a>,
  ) -> Option<BindingIdentifier<'a>> {
    let symbol = node.symbol_id.get().unwrap();
    self.update_var_decl_state(symbol, true);

    let referred = self.is_referred(AstKind2::BindingIdentifier(&node));
    referred.then(|| self.clone_node(node))
  }
}
