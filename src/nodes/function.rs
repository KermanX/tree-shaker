use crate::{
  entity::{arguments::ArgumentsEntity, function::FunctionEntity, Entity},
  TreeShaker,
};
use oxc::ast::ast::Function;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_function(&mut self, node: &'a Function) -> Entity {
    self.functions.insert(node.span, node);
    Entity::Function(FunctionEntity::new(node.span))
  }

  pub(crate) fn call_function(
    &mut self,
    node: &'a Function,
    this: Entity,
    args: ArgumentsEntity,
  ) -> (bool, Entity) {
    todo!()
  }
}
