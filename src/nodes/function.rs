use crate::{
  entity::{function::FunctionEntity, Entity},
  symbol::{arguments::ArgumentsEntity, SymbolSource},
  Analyzer,
};
use oxc::ast::ast::Function;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_function(&mut self, node: &'a Function) -> (bool, Entity) {
    if let Some(id) = &node.id {
      self.declare_symbol(SymbolSource::Function(node), id.symbol_id.get().unwrap());
    }
    (false, Entity::Function(FunctionEntity::new(node.span)))
  }

  pub(crate) fn calc_function(&self, node: &'a Function<'a>) -> Entity {
    Entity::Function(FunctionEntity::new(node.span))
  }

  pub(crate) fn call_function(
    &mut self,
    node: &'a Function<'a>,
    this: Entity,
    args: ArgumentsEntity<'a>,
  ) -> (bool, Entity) {
    self.exec_formal_parameters(&node.params, args);
    todo!()
  }
}
