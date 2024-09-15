use crate::entity::{
  dep::EntityDepNode, entity::Entity, forwarded::ForwardedEntity, unknown::UnknownEntity,
};

pub fn create_environment<'a>() -> Entity<'a> {
  ForwardedEntity::new(UnknownEntity::new_unknown(), EntityDepNode::Environment)
}
