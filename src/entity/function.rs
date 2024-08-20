use super::{arguments::ArgumentsEntity, Entity};
use oxc::span::Span;

#[derive(Debug, Default, Clone)]
pub struct FunctionEntity {}

impl FunctionEntity {
  pub fn new(span: Span) -> Self {
    FunctionEntity {}
  }

  pub(crate) fn call(&self, this: Entity, args: ArgumentsEntity) -> (bool, Entity) {
    todo!()
  }
}
