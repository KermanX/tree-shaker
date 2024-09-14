use super::{
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  typeof_result::TypeofResult,
  unknown::UnknownEntity,
};
use crate::analyzer::Analyzer;

#[derive(Debug)]
pub struct ArgumentsEntity<'a> {
  pub arguments: Vec<(bool, Entity<'a>)>,
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

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: EntityDep,
    _key: &Entity<'a>,
  ) -> Entity<'a> {
    unreachable!()
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: EntityDep,
    _key: &Entity<'a>,
    _value: Entity<'a>,
  ) {
    unreachable!()
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    unreachable!()
  }

  fn delete_property(&self, _analyzer: &mut Analyzer<'a>, _key: &Entity<'a>) -> bool {
    unreachable!()
  }

  fn call(
    &self,
    _analyzer: &mut Analyzer<'a>,
    _dep: EntityDep,
    _this: &Entity<'a>,
    _args: &Entity<'a>,
  ) -> Entity<'a> {
    unreachable!()
  }

  fn r#await(&self, _rc: &Entity<'a>, _analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    unreachable!()
  }

  fn iterate(&self, _rc: &Entity<'a>, _analyzer: &mut Analyzer<'a>) -> (bool, Option<Entity<'a>>) {
    unreachable!()
  }

  fn get_typeof(&self) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_array(&self, _rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
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
  pub fn new(arguments: Vec<(bool, Entity<'a>)>) -> Entity<'a> {
    Entity::new(Self { arguments })
  }
}
