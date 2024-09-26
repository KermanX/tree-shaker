use super::{
  consumable::Consumable,
  consumed_object,
  entity::{Entity, EntityTrait},
  interactions::InteractionKind,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::UnknownEntity,
};
use crate::analyzer::Analyzer;

#[derive(Debug, Clone)]
pub struct PromiseEntity<'a> {
  pub value: Entity<'a>,
  pub errors: Option<Vec<Entity<'a>>>,
  pub call_dep: Consumable<'a>,
}

impl<'a> EntityTrait<'a> for PromiseEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.value.consume(analyzer);
    if let Some(errors) = &self.errors {
      for error in errors {
        error.consume(analyzer);
      }
    }
    analyzer.consume(self.call_dep.clone());
  }

  fn interact(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, kind: InteractionKind) {
    self.consume(analyzer);
    consumed_object::interact(analyzer, dep, kind)
  }

  fn get_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    analyzer.builtins.prototypes.promise.get_property(rc, key, dep)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
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
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    self.consume(analyzer);
    consumed_object::enumerate_properties(analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: &Entity<'a>) {
    self.consume(analyzer);
    consumed_object::delete_property(analyzer, dep, key)
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.consume(analyzer);
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    if let Some(errors) = &self.errors {
      analyzer.forward_throw(errors.clone(), self.call_dep.clone());
    }
    self.value.r#await(analyzer)
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("object")
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_computed_string(self.value.clone())
  }

  fn get_to_numeric(&self, rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_computed_unknown(vec![rc.clone()])
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_computed_string(self.value.clone())
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
  pub fn new(
    value: Entity<'a>,
    errors: Option<Vec<Entity<'a>>>,
    call_dep: Consumable<'a>,
  ) -> Entity<'a> {
    Entity::new(Self { value, errors, call_dep })
  }
}
