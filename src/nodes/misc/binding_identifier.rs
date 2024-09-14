use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  entity::{entity::Entity, forwarded::ForwardedEntity},
  transformer::Transformer,
};
use oxc::ast::{ast::BindingIdentifier, AstKind};

impl<'a> Analyzer<'a> {
  pub fn declare_binding_identifier(
    &mut self,
    node: &'a BindingIdentifier<'a>,
    exporting: bool,
    kind: DeclarationKind,
  ) {
    let symbol = node.symbol_id.get().unwrap();
    let dep = AstKind::BindingIdentifier(node);
    self.declare_symbol(symbol, dep, exporting, kind, None);
  }

  pub fn init_binding_identifier(&mut self, node: &'a BindingIdentifier<'a>, init: Entity<'a>) {
    let symbol = node.symbol_id.get().unwrap();
    let dep = AstKind::BindingIdentifier(node);
    self.init_symbol(symbol, ForwardedEntity::new(init, dep));
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_binding_identifier(
    &self,
    node: &'a BindingIdentifier<'a>,
  ) -> Option<BindingIdentifier<'a>> {
    let referred = self.is_referred(AstKind::BindingIdentifier(&node));
    referred.then(|| self.clone_node(node))
  }
}
