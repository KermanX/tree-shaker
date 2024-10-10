use super::{consumed_object, Consumable, Entity, EntityFactory, EntityTrait, TypeofResult};
use crate::analyzer::Analyzer;

#[derive(Debug, Clone)]
pub struct PromiseEntity<'a> {
  pub value: Entity<'a>,
  pub errors: Option<Vec<Entity<'a>>>,
}

impl<'a> EntityTrait<'a> for PromiseEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.value.consume(analyzer);
    if let Some(errors) = &self.errors {
      for error in errors {
        error.consume(analyzer);
      }
    }
  }

  fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    analyzer.builtins.prototypes.promise.get_property(analyzer, rc, key, dep)
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.consume(analyzer);
    consumed_object::set_property(analyzer, dep, key, value)
  }

  fn enumerate_properties(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    if analyzer.config.unknown_property_read_side_effects {
      self.consume(analyzer);
    }
    consumed_object::enumerate_properties(rc, analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.consume(analyzer);
    consumed_object::delete_property(analyzer, dep, key)
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.consume(analyzer);
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    if let Some(errors) = &self.errors {
      analyzer.forward_throw(errors.clone());
    }
    self.value.r#await(analyzer, dep)
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.new_string("object")
  }

  fn get_to_string(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.new_computed_unknown_string(self.value.to_consumable())
  }

  fn get_to_numeric(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.new_computed_unknown(rc.to_consumable())
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.new_boolean(true)
  }

  fn get_to_property_key(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.new_computed_unknown_string(self.value.to_consumable())
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

impl<'a> EntityFactory<'a> {
  pub fn new_promise(&self, value: Entity<'a>, errors: Option<Vec<Entity<'a>>>) -> Entity<'a> {
    self.new_entity(PromiseEntity { value, errors })
  }
}
