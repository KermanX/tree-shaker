use crate::{
  consumable::box_consumable,
  entity::{Entity, EntityDepNode, ForwardedEntity, UnknownEntity},
};

pub fn create_environment<'a>() -> Entity<'a> {
  ForwardedEntity::new(UnknownEntity::new_unknown(), box_consumable(EntityDepNode::Environment))
}
