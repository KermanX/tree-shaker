use super::{
  entity::Entity,
  literal::LiteralEntity,
  union::UnionEntity,
  unknown::{UnknownEntity, UnknownEntityKind},
};

pub fn collect_effect_and_value<'a>(values: Vec<(bool, Entity<'a>)>) -> (bool, Entity<'a>) {
  let mut has_effect = false;
  let mut result = Vec::new();
  for (effect, value) in values {
    has_effect |= effect;
    result.push(value);
  }
  (has_effect, UnionEntity::new(result))
}

pub fn boolean_from_test_result<'a>(result: Option<bool>) -> Entity<'a> {
  match result {
    Some(value) => LiteralEntity::new_boolean(value),
    None => UnknownEntity::new(UnknownEntityKind::Boolean),
  }
}
