use super::{
  consumed_object,
  dep::EntityDep,
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

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    analyzer.builtins.prototypes.promise.get_property(key, dep)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    self.consume_as_unknown(analyzer);
    consumed_object::set_property(analyzer, dep, key, value)
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    self.consume_as_unknown(analyzer);
    consumed_object::enumerate_properties(analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
    self.consume_as_unknown(analyzer);
    consumed_object::delete_property(analyzer, key)
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.consume_as_unknown(analyzer);
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    let (inner_effect, awaited) = self.value.r#await(analyzer);
    (self.has_effect || inner_effect, awaited)
  }

  fn iterate(&self, _rc: &Entity<'a>, _analyzer: &mut Analyzer<'a>) -> (bool, Option<Entity<'a>>) {
    // TODO: throw warning
    (true, Some(UnknownEntity::new_unknown()))
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
