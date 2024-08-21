use crate::{entity::Entity, Analyzer};
use oxc::{ast::ast::VariableDeclarator, semantic::SymbolId};

use super::binding_pattern::BindingPatternSource;

#[derive(Debug, Default, Clone)]
pub struct Data {
  init_val: Entity,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_variable_declarator(&mut self, node: &'a VariableDeclarator) {
    let init_val = match &node.init {
      Some(init) => self.exec_expression(init),
      None => Entity::Undefined,
    };

    self.exec_binding_pattern(&node.id, BindingPatternSource::VariableDeclarator(node));

    self.set_data(node, Data { init_val });
  }

  pub(crate) fn calc_variable_declarator(
    &self,
    node: &'a VariableDeclarator,
    symbol: SymbolId,
  ) -> Entity {
    let data = self.get_data::<Data>(node);
    todo!()
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
