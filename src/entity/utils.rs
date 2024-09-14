use super::{
  entity::Entity,
  literal::LiteralEntity,
  union::UnionEntity,
  unknown::{UnknownEntity, UnknownEntityKind},
};

pub fn boolean_from_test_result<'a>(
  result: Option<bool>,
  deps: impl FnOnce() -> Vec<Entity<'a>>,
) -> Entity<'a> {
  match result {
    Some(value) => LiteralEntity::new_boolean(value),
    None => UnknownEntity::new_with_deps(UnknownEntityKind::Boolean, deps()),
  }
}

#[macro_export]
macro_rules! use_consumed_flag {
  ($self: expr) => {
    if $self.consumed.get() {
      return;
    }
    $self.consumed.set(true);
  };
}
