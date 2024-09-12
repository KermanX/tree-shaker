use crate::entity::{
  dep::{EntityDep, EntityDepNode},
  entity::Entity,
  forwarded::ForwardedEntity,
  unknown::UnknownEntity,
};

pub fn create_environment<'a>() -> Entity<'a> {
  ForwardedEntity::new(
    UnknownEntity::new_unknown(),
    EntityDep { node: EntityDepNode::Environment, scope_path: vec![] },
  )
}
