use super::{
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableTrait},
  use_consumed_flag,
};
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

    analyzer.consume(self.val);
    analyzer.consume(self.dep);
  }

  fn consume_mangable(&self, analyzer: &mut Analyzer<'a>) -> bool {
    if !self.consumed.get() {
      analyzer.consume(self.dep);
      let consumed = self.val.consume_mangable(analyzer);
      self.consumed.set(consumed);
      consumed
    } else {
      true
    }
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    self.val.unknown_mutate(analyzer, self.forward_dep(dep, analyzer));
  }

  fn get_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    self.val.get_property(analyzer, self.forward_dep(dep, analyzer), key)
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.val.set_property(analyzer, self.forward_dep(dep, analyzer), key, value);
  }

  fn enumerate_properties(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    self.val.enumerate_properties(analyzer, self.forward_dep(dep, analyzer))
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.val.delete_property(analyzer, self.forward_dep(dep, analyzer), key)
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.val.call(analyzer, self.forward_dep(dep, analyzer), this, args)
  }

  fn construct(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.val.construct(analyzer, self.forward_dep(dep, analyzer), args)
  }

  fn jsx(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    self.forward_value(self.val.jsx(analyzer, props), analyzer)
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    self.val.r#await(analyzer, self.forward_dep(dep, analyzer))
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    self.val.iterate(analyzer, self.forward_dep(dep, analyzer))
  }

  fn get_destructable(
    &self,
    _rc: Entity<'a>,
    analyzer: &Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Consumable<'a> {
    self.val.get_destructable(analyzer, self.forward_dep(dep, analyzer))
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_typeof(analyzer), analyzer)
  }

  fn get_to_string(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_to_string(analyzer), analyzer)
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_to_numeric(analyzer), analyzer)
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_to_boolean(analyzer), analyzer)
  }

  fn get_to_property_key(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_to_property_key(analyzer), analyzer)
  }

  fn get_to_jsx_child(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_to_jsx_child(analyzer), analyzer)
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
  pub fn forward_dep(&self, dep: Consumable<'a>, analyzer: &Analyzer<'a>) -> Consumable<'a> {
    if self.consumed.get() {
      dep
    } else {
      analyzer.consumable((self.dep, dep))
    }
  }

  pub fn forward_value(&self, val: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.computed(val, self.dep)
  }
}

impl<'a> EntityFactory<'a> {
  pub fn computed(&self, val: Entity<'a>, dep: impl ConsumableTrait<'a> + 'a) -> Entity<'a> {
    self.entity(ComputedEntity { val, dep: self.consumable(dep), consumed: Cell::new(false) })
  }

  pub fn optional_computed(
    &self,
    val: Entity<'a>,
    dep: Option<impl ConsumableTrait<'a> + 'a>,
  ) -> Entity<'a> {
    match dep {
      Some(dep) => self.computed(val, dep),
      None => val,
    }
  }
}
