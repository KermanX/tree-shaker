use crate::{entity::Entity, symbol::SymbolSource, TreeShaker};
use oxc::ast::ast::FormalParameter;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_formal_parameter(&mut self, node: &'a FormalParameter<'a>, arg: Entity) {
    let data = self.load_data::<Data>(node);

    self.exec_binding_pattern(&node.pattern, SymbolSource::FormalParameter(node, arg));
  }
}
