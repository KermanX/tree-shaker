use crate::entity::{Entity, EntityDepNode, ForwardedEntity, UnknownEntity};

pub fn create_environment<'a>() -> Entity<'a> {
  ForwardedEntity::new(UnknownEntity::new_unknown(), EntityDepNode::Environment)
}
