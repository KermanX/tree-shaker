use super::{
  entity::{Entity, EntityTrait},
  typeof_result::TypeofResult,
  unknown::UnknownEntity,
};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct ArgumentsEntity<'a> {
  pub(crate) arguments: Vec<(bool, Entity<'a>)>,
}

impl<'a> EntityTrait<'a> for ArgumentsEntity<'a> {
  fn consume_self(&self, _analyzer: &mut crate::analyzer::Analyzer<'a>) {
    unreachable!()
  }

  fn consume_as_unknown(&self, analyzer: &mut crate::analyzer::Analyzer<'a>) {
    for (_, entity) in &self.arguments {
      entity.consume_as_unknown(analyzer);
    }
  }

  fn get_property(&self, _key: &Entity<'a>) -> (bool, Entity<'a>) {
    unreachable!()
  }

  fn set_property(&self, key: &Entity<'a>, value: Entity<'a>) -> bool {
    unreachable!()
  }

  fn get_typeof(&self) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_string(&self) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_property_key(&self) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_array(&self, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    let mut result = Vec::new();
    for i in 0..length.min(self.arguments.len()) {
      let (is_spread, entity) = &self.arguments[i];
      assert!(!is_spread, "TODO:");
      result.push(entity.clone());
    }
    for _ in 0..length.saturating_sub(self.arguments.len()) {
      result.push(UnknownEntity::new_unknown());
    }
    (result, UnknownEntity::new_unknown())
  }

  fn test_typeof(&self) -> TypeofResult {
    unreachable!()
  }

  fn test_truthy(&self) -> Option<bool> {
    unreachable!()
  }

  fn test_nullish(&self) -> Option<bool> {
    unreachable!()
  }
}

impl<'a> ArgumentsEntity<'a> {
  pub(crate) fn new(arguments: Vec<(bool, Entity<'a>)>) -> Entity<'a> {
    Rc::new(Self { arguments })
  }
}
