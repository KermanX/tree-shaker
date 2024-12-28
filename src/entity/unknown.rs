use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableTrait},
};
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct UnknownEntity<'a>(PhantomData<&'a ()>);

impl<'a> EntityTrait<'a> for UnknownEntity<'a> {
  fn consume(&self, _analyzer: &mut Analyzer<'a>) {}

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    consumed_object::unknown_mutate(analyzer, dep)
  }

  fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    consumed_object::get_property(rc, analyzer, dep, key)
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
  ) -> EnumeratedProperties<'a> {
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
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    consumed_object::call(rc, analyzer, dep, this, args)
  }

  fn construct(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    consumed_object::construct(rc, analyzer, dep, args)
  }

  fn jsx(&self, rc: Entity<'a>, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    consumed_object::jsx(rc, analyzer, props)
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    self.consume(analyzer);
    consumed_object::r#await(analyzer, dep)
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_destructable(
    &self,
    rc: Entity<'a>,
    analyzer: &Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Consumable<'a> {
    analyzer.consumable((rc, dep))
  }

  fn get_typeof(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.computed_unknown_string(rc)
  }

  fn get_to_string(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.computed_unknown_string(rc)
  }

  fn get_to_numeric(&self, rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    rc
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    match self.test_truthy() {
      Some(val) => analyzer.factory.boolean(val),
      None => analyzer.factory.unknown_boolean,
    }
  }

  fn get_to_property_key(&self, rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    rc
  }

  fn get_to_jsx_child(&self, rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    rc
  }

  fn test_typeof(&self) -> TypeofResult {
    TypeofResult::_Unknown
  }

  fn test_truthy(&self) -> Option<bool> {
    None
  }

  fn test_nullish(&self) -> Option<bool> {
    None
  }
}

impl<'a> UnknownEntity<'a> {
  pub fn new() -> Self {
    Self::default()
  }
}

impl<'a> EntityFactory<'a> {
  pub fn unknown(&self) -> Entity<'a> {
    self.immutable_unknown
  }

  pub fn computed_unknown(&self, dep: impl ConsumableTrait<'a> + 'a) -> Entity<'a> {
    self.computed(self.immutable_unknown, dep)
  }
}
