use crate::{
  entity::{function::FunctionEntity, Entity}, symbol::{arguments::ArgumentsEntity, SymbolSource}, transformer::Transformer, Analyzer
};
use oxc::ast::ast::Function;

#[derive(Debug, Default, Clone)]
pub struct Data {
  referred: bool,
}

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

  pub(crate) fn refer_function(&mut self, node: &'a Function<'a>) {
    let data = self.load_data::<Data>(node);
    data.referred = true;
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

impl<'a> Transformer<'a> {
  pub fn transform_function(&self, node: Function<'a>) -> Option<Function<'a>> {
    let data = self.get_data::<Data>(&node);

    if !data.referred {
      return None;
    }

    todo!()
  }
}
