use super::{
  consumed_object,
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  interactions::InteractionKind,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use crate::analyzer::Analyzer;

#[derive(Debug, Clone)]
pub struct PromiseEntity<'a> {
  pub has_effect: bool,
  pub value: Entity<'a>,
  pub error: Option<Entity<'a>>,
}

impl<'a> EntityTrait<'a> for PromiseEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.value.consume(analyzer);
    self.error.as_ref().map(|e| e.consume(analyzer));
  }

  fn interact(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep, kind: InteractionKind) {
    self.consume(analyzer);
    consumed_object::interact(analyzer, dep, kind)
  }

  fn get_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    analyzer.builtins.prototypes.promise.get_property(rc, key, dep)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    self.consume(analyzer);
    consumed_object::set_property(analyzer, dep, key, value)
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    self.consume(analyzer);
    consumed_object::enumerate_properties(analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep, key: &Entity<'a>) {
    self.consume(analyzer);
    consumed_object::delete_property(analyzer, dep, key)
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.consume(analyzer);
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    if let Some(error) = &self.error {
      analyzer.try_scope_mut().throw(error.clone());
    }
    let (inner_effect, awaited) = self.value.r#await(analyzer);
    (self.has_effect || inner_effect || self.error.is_some(), awaited)
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("object")
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![self.value.clone()])
  }

  fn get_to_numeric(&self, rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_unknown_with_deps(vec![rc.clone()])
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![self.value.clone()])
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
  pub fn new(has_effect: bool, value: Entity<'a>, error: Option<Entity<'a>>) -> Entity<'a> {
    Entity::new(Self { has_effect, value, error })
  }
}
