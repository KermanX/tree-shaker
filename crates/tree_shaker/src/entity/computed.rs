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
pub struct ComputedEntity<'a, T: ConsumableTrait<'a> + Copy + 'a> {
  val: Entity<'a>,
  dep: T,
  consumed: Cell<bool>,
}

impl<'a, T: ConsumableTrait<'a> + Copy + 'a> EntityTrait<'a> for ComputedEntity<'a, T> {
  fn consume(&'a self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    analyzer.consume(self.val);
    analyzer.consume(self.dep);
  }

  fn consume_mangable(&'a self, analyzer: &mut Analyzer<'a>) -> bool {
    if !self.consumed.get() {
      analyzer.consume(self.dep);
      let consumed = self.val.consume_mangable(analyzer);
      self.consumed.set(consumed);
      consumed
    } else {
      true
    }
  }

  fn unknown_mutate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    self.val.unknown_mutate(analyzer, self.forward_dep(dep, analyzer));
  }

  fn get_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    self.val.get_property(analyzer, self.forward_dep(dep, analyzer), key)
  }

  fn set_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.val.set_property(analyzer, self.forward_dep(dep, analyzer), key, value);
  }

  fn enumerate_properties(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    self.val.enumerate_properties(analyzer, self.forward_dep(dep, analyzer))
  }

  fn delete_property(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.val.delete_property(analyzer, self.forward_dep(dep, analyzer), key)
  }

  fn call(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.val.call(analyzer, self.forward_dep(dep, analyzer), this, args)
  }

  fn construct(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.val.construct(analyzer, self.forward_dep(dep, analyzer), args)
  }

  fn jsx(&'a self, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    self.forward_value(self.val.jsx(analyzer, props), analyzer)
  }

  fn r#await(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> Entity<'a> {
    self.val.r#await(analyzer, self.forward_dep(dep, analyzer))
  }

  fn iterate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> IteratedElements<'a> {
    self.val.iterate(analyzer, self.forward_dep(dep, analyzer))
  }

  fn get_destructable(&'a self, analyzer: &Analyzer<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    self.val.get_destructable(analyzer, self.forward_dep(dep, analyzer))
  }

  fn get_typeof(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_typeof(analyzer), analyzer)
  }

  fn get_to_string(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_to_string(analyzer), analyzer)
  }

  fn get_to_numeric(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_to_numeric(analyzer), analyzer)
  }

  fn get_to_boolean(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_to_boolean(analyzer), analyzer)
  }

  fn get_to_property_key(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_to_property_key(analyzer), analyzer)
  }

  fn get_to_jsx_child(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward_value(self.val.get_to_jsx_child(analyzer), analyzer)
  }

  fn get_to_literals(&'a self, analyzer: &Analyzer<'a>) -> Option<FxHashSet<LiteralEntity<'a>>> {
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

impl<'a, T: ConsumableTrait<'a> + Copy + 'a> ComputedEntity<'a, T> {
  fn forward_dep(&self, dep: Consumable<'a>, analyzer: &Analyzer<'a>) -> Consumable<'a> {
    if self.consumed.get() {
      dep
    } else {
      analyzer.factory.consumable_no_once((self.dep, dep))
    }
  }

  fn forward_value(&self, val: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if self.consumed.get() {
      val
    } else {
      analyzer.factory.computed(val, self.dep)
    }
  }
}

impl<'a> EntityFactory<'a> {
  pub fn computed<T: ConsumableTrait<'a> + Copy + 'a>(
    &self,
    val: Entity<'a>,
    dep: T,
  ) -> Entity<'a> {
    self.alloc(ComputedEntity { val, dep, consumed: Cell::new(false) })
  }

  pub fn optional_computed(
    &self,
    val: Entity<'a>,
    dep: Option<impl ConsumableTrait<'a> + Copy + 'a>,
  ) -> Entity<'a> {
    match dep {
      Some(dep) => self.computed(val, dep),
      None => val,
    }
  }
}
