use super::{
  computed::ComputedEntity, consumable::Consumable, entity::Entity, literal::LiteralEntity,
  unknown::UnknownEntity,
};

pub fn boolean_from_test_result<'a, T: Into<Consumable<'a>>>(
  result: Option<bool>,
  deps: impl FnOnce() -> T,
) -> Entity<'a> {
  match result {
    Some(value) => LiteralEntity::new_boolean(value),
    None => ComputedEntity::new(UnknownEntity::new_boolean(), deps()),
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
