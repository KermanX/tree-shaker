use super::{
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use crate::analyzer::Analyzer;

#[derive(Debug, Clone)]
pub struct PromiseEntity<'a> {
  pub has_effect: bool,
  pub value: Entity<'a>,
}

impl<'a> EntityTrait<'a> for PromiseEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {
    self.value.consume_as_unknown(analyzer);
  }

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    self.value.consume_as_unknown(analyzer);
  }

  fn get_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> (bool, Entity<'a>) {
    todo!("built-ins & extra properties")
  }

  fn set_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool {
    todo!("built-ins & extra properties")
  }

  fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
    self.consume_as_unknown(analyzer);
    UnknownEntity::new_unknown_to_entries_result(vec![])
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
    self.consume_as_unknown(analyzer);
    key.consume_self(analyzer);
    true
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    self.consume_as_unknown(analyzer);
    this.consume_as_unknown(analyzer);
    args.consume_as_unknown(analyzer);
    (true, UnknownEntity::new_unknown())
  }

  fn r#await(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    let (inner_effect, awaited) = self.value.r#await(analyzer);
    (self.has_effect || inner_effect, awaited)
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("object")
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![self.value.clone()])
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![self.value.clone()])
  }

  fn get_to_array(&self, _rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    UnknownEntity::new_unknown_to_array_result(length, vec![self.value.clone()])
  }

  fn test_typeof(&self) -> TypeofResult {
    TypeofResult::Object
  }

  fn test_truthy(&self) -> Option<bool> {
    Some(true)
  }

  fn test_nullish(&self) -> Option<bool> {
    Some(false)
  }
}

impl<'a> PromiseEntity<'a> {
  pub fn new(has_effect: bool, value: Entity<'a>) -> Entity<'a> {
    Entity::new(Self { has_effect, value })
  }
}
