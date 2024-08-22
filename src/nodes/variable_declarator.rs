use crate::{entity::Entity, Analyzer};
use oxc::{ast::ast::VariableDeclarator, semantic::SymbolId};

use super::binding_pattern::BindingPatternSource;

#[derive(Debug, Default, Clone)]
pub struct Data {
  init_effect: bool,
  init_val: Entity,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_variable_declarator(&mut self, node: &'a VariableDeclarator) -> bool {
    let (init_effect, init_val) = match &node.init {
      Some(init) => self.exec_expression(init),
      None => (false, Entity::Undefined),
    };

    self.exec_binding_pattern(
      &node.id,
      BindingPatternSource::VariableDeclarator(node),
      init_val.clone(),
    );

    self.set_data(node, Data { init_effect, init_val });

    init_effect
  }

  pub(crate) fn calc_variable_declarator(
    &self,
    node: &'a VariableDeclarator,
    symbol: SymbolId,
  ) -> Entity {
    self.calc_binding_pattern(&node.id, symbol).unwrap()
  }

  pub(crate) fn refer_variable_declarator(
    &mut self,
    node: &'a VariableDeclarator,
    symbol: SymbolId,
  ) {
    self.refer_binding_pattern(&node.id, symbol)
  }
}
