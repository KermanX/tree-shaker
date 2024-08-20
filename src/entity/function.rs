use super::{arguments::ArgumentsEntity, Entity};
use crate::TreeShaker;
use oxc::span::Span;

#[derive(Debug, Default, Clone)]
pub struct FunctionEntity {
  span: Span,
}

impl FunctionEntity {
  pub fn new(span: Span) -> Self {
    FunctionEntity { span }
  }

  pub(crate) fn call<'a>(
    &self,
    tree_shaker: &mut TreeShaker<'a>,
    this: Entity,
    args: ArgumentsEntity,
  ) -> (bool, Entity) {
    let node = tree_shaker.functions.get(&self.span).unwrap();
    tree_shaker.call_function(node, this, args)
  }
}
