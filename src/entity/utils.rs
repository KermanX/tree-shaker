use super::{entity::Entity, union::UnionEntity};

pub(crate) fn collect_effect_and_value<'a>(values: Vec<(bool, Entity<'a>)>) -> (bool, Entity<'a>) {
  let mut has_effect = false;
  let mut result = Vec::new();
  for (effect, value) in values {
    has_effect |= effect;
    result.push(value);
  }
  (has_effect, UnionEntity::new(result))
}
