use crate::{
  analyzer::Analyzer,
  entity::{dep::EntityDepNode, entity::Entity, forwarded::ForwardedEntity},
  transformer::Transformer,
};
use oxc::ast::ast::BindingIdentifier;

impl<'a> Analyzer<'a> {
  pub fn exec_binding_identifier(
    &mut self,
    node: &'a BindingIdentifier<'a>,
    init: Entity<'a>,
    exporting: bool,
  ) {
    let symbol = node.symbol_id.get().unwrap();
    let dep = self.new_entity_dep(EntityDepNode::BindingIdentifier(node));
    self.declare_symbol(symbol, dep.clone(), ForwardedEntity::new(init, dep), exporting);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_binding_identifier(
    &mut self,
    node: BindingIdentifier<'a>,
  ) -> Option<BindingIdentifier<'a>> {
    let referred = self.is_referred(EntityDepNode::BindingIdentifier(&node));
    referred.then_some(node)
  }
}
