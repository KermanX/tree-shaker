use crate::entity::{
  dep::ENVIRONMENT_DEP, entity::Entity, forwarded::ForwardedEntity, unknown::UnknownEntity,
};

pub fn create_environment<'a>() -> Entity<'a> {
  ForwardedEntity::new(UnknownEntity::new_unknown(), ENVIRONMENT_DEP)
}
