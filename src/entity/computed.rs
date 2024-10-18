use super::{
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable, ConsumableTrait},
  use_consumed_flag,
};
use rustc_hash::FxHashSet;
use std::cell::Cell;

#[derive(Debug)]
pub struct ComputedEntity<'a, T: ConsumableTrait<'a> + 'a> {
  val: Entity<'a>,
  dep: T,
  consumed: Cell<bool>,
}

impl<'a, T: ConsumableTrait<'a> + 'a> EntityTrait<'a> for ComputedEntity<'a, T> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    self.val.consume(analyzer);
    self.dep.consume(analyzer);
  }

  fn get_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    self.val.get_property(analyzer, box_consumable((self.dep.cloned(), dep)), key)
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.val.set_property(analyzer, box_consumable((self.dep.cloned(), dep)), key, value);
  }

  fn enumerate_properties(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    self.val.enumerate_properties(analyzer, box_consumable((self.dep.cloned(), dep)))
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.val.delete_property(analyzer, box_consumable((self.dep.cloned(), dep)), key)
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.val.call(analyzer, box_consumable((self.dep.cloned(), dep)), this, args)
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    self.val.r#await(analyzer, box_consumable((self.dep.cloned(), dep)))
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    self.val.iterate(analyzer, box_consumable((self.dep.cloned(), dep)))
  }

  fn get_destructable(&self, _rc: Entity<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    self.val.get_destructable(box_consumable((self.dep.cloned(), dep)))
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_typeof(analyzer), analyzer)
  }

  fn get_to_string(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_string(analyzer), analyzer)
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_numeric(analyzer), analyzer)
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_boolean(analyzer), analyzer)
  }

  fn get_to_property_key(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_property_key(analyzer), analyzer)
  }

  fn get_to_literals(
    &self,
    _rc: Entity<'a>,
    analyzer: &Analyzer<'a>,
  ) -> Option<FxHashSet<LiteralEntity<'a>>> {
    self.val.get_to_literals(analyzer)
  }

  fn test_typeof(&self) -> TypeofResult {
    self.val.test_typeof()
  }

  fn test_truthy(&self) -> Option<bool> {
    self.val.test_truthy()
  }

  fn test_nullish(&self) -> Option<bool> {
    self.val.test_nullish()
  }
}

impl<'a, T: ConsumableTrait<'a> + 'a> ComputedEntity<'a, T> {
  pub fn forward(&self, val: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.computed(val, self.dep.cloned())
  }
}

impl<'a> EntityFactory<'a> {
  pub fn computed<T: ConsumableTrait<'a> + 'a>(&self, val: Entity<'a>, dep: T) -> Entity<'a> {
    self.entity(ComputedEntity { val, dep, consumed: Cell::new(false) })
  }
}
