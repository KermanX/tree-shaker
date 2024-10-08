use super::{Consumable, Entity, EntityFactory, EntityTrait, LiteralEntity, TypeofResult};
use crate::{analyzer::Analyzer, use_consumed_flag};
use rustc_hash::FxHashSet;
use std::cell::Cell;

#[derive(Debug)]
pub struct ComputedEntity<'a> {
  val: Entity<'a>,
  dep: Consumable<'a>,
  consumed: Cell<bool>,
}

impl<'a> EntityTrait<'a> for ComputedEntity<'a> {
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
    self.val.get_property(analyzer, (self.dep.clone(), dep), key)
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.val.set_property(analyzer, (self.dep.clone(), dep), key, value);
  }

  fn enumerate_properties(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    self.val.enumerate_properties(analyzer, (self.dep.clone(), dep))
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.val.delete_property(analyzer, (self.dep.clone(), dep), key)
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.val.call(analyzer, (self.dep.clone(), dep), this, args)
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    self.val.r#await(analyzer, (self.dep.clone(), dep))
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    self.val.iterate(analyzer, (self.dep.clone(), dep))
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

impl<'a> ComputedEntity<'a> {
  pub fn forward(&self, val: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.new_computed(val, self.dep.clone())
  }
}

impl<'a> EntityFactory<'a> {
  pub fn new_computed(&self, val: Entity<'a>, dep: impl Into<Consumable<'a>>) -> Entity<'a> {
    self.new_entity(ComputedEntity { val, dep: dep.into(), consumed: Cell::new(false) })
  }
}
