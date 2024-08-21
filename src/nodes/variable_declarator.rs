use crate::{entity::Entity, TreeShaker};
use oxc::{ast::ast::VariableDeclarator, semantic::SymbolId};

use super::binding_pattern::BindingPatternSource;

#[derive(Debug, Default, Clone)]
pub struct Data {
  init_val: Entity,
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_variable_declarator(&mut self, node: &'a VariableDeclarator) {
    let data = self.load_data::<Data>(node);

    data.init_val = match &node.init {
      Some(init) => self.exec_expression(init),
      None => Entity::Undefined,
    };

    self.exec_binding_pattern(&node.id, BindingPatternSource::VariableDeclarator(node));
  }

  pub(crate) fn refer_variable_declarator(
    &mut self,
    node: &'a VariableDeclarator,
    symbol: SymbolId,
  ) -> Entity {
    let data = self.load_data::<Data>(node);

    self.refer_binding_pattern(&node.id, symbol, data.init_val.clone())
  }
}
