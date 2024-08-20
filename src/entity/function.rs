use oxc::span::Span;
use super::Entity;

#[derive(Debug, Default, Clone)]
pub struct FunctionEntity {}

impl FunctionEntity {
  pub fn new(span: Span) -> Self {
    
    FunctionEntity {}
  }

  pub(crate) fn call(&self, this: Option<&Entity>, args: &[Entity]) -> Entity {
    Entity::Unknown
  }
}
