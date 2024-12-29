use super::{
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{analyzer::Analyzer, consumable::Consumable, use_consumed_flag};
use rustc_hash::FxHashSet;
use std::cell::{Cell, RefCell};

#[derive(Debug)]
pub struct CollectedEntity<'a> {
  val: Entity<'a>,
  deps: &'a RefCell<Vec<Entity<'a>>>,
  consumed: Cell<bool>,
}

impl<'a> EntityTrait<'a> for CollectedEntity<'a> {
  fn consume(&'a self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);
    self.consume_deps(analyzer);
    self.val.consume(analyzer);
  }

  fn consume_mangable(&'a self, analyzer: &mut Analyzer<'a>) -> bool {
    self.consume_deps(analyzer);
    self.val.consume_mangable(analyzer)
  }

  fn unknown_mutate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    self.val.unknown_mutate(analyzer, dep)
  }

  fn get_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    let value = self.val.get_property(analyzer, dep, key);
    self.forward(value, analyzer)
  }

  fn set_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.consume_deps(analyzer);
    self.val.set_property(analyzer, dep, key, value)
  }

  fn enumerate_properties(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    self.consume_deps(analyzer);
    self.val.enumerate_properties(analyzer, dep)
  }

  fn delete_property(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.consume_deps(analyzer);
    self.val.delete_property(analyzer, dep, key)
  }

  fn call(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let value = self.val.call(analyzer, dep, this, args);
    self.forward(value, analyzer)
  }

  fn construct(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let value = self.val.construct(analyzer, dep, args);
    self.forward(value, analyzer)
  }

  fn jsx(&'a self, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    analyzer.factory.computed(self.val.jsx(analyzer, props), self.deps)
  }

  fn r#await(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> Entity<'a> {
    self.forward(self.val.r#await(analyzer, dep), analyzer)
  }

  fn iterate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> IteratedElements<'a> {
    let (elements, rest, deps) = self.val.iterate(analyzer, dep);
    (elements, rest, analyzer.consumable((deps, self.deps)))
  }

  fn get_destructable(&'a self, analyzer: &Analyzer<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    analyzer.consumable((self.deps, dep))
  }

  fn get_typeof(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    // TODO: Verify this
    self.forward(self.val.get_typeof(analyzer), analyzer)
  }

  fn get_to_string(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_string(analyzer), analyzer)
  }

  fn get_to_numeric(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_numeric(analyzer), analyzer)
  }

  fn get_to_boolean(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_boolean(analyzer), analyzer)
  }

  fn get_to_property_key(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_property_key(analyzer), analyzer)
  }

  fn get_to_jsx_child(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_jsx_child(analyzer), analyzer)
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

impl<'a> CollectedEntity<'a> {
  fn forward(&self, val: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if self.consumed.get() {
      val
    } else {
      analyzer.factory.collected(val, self.deps)
    }
  }

  fn consume_deps(&self, analyzer: &mut Analyzer<'a>) {
    for entity in self.deps.take().iter() {
      entity.consume_mangable(analyzer);
    }
  }
}

impl<'a> EntityFactory<'a> {
  pub fn collected(&self, val: Entity<'a>, collected: &'a RefCell<Vec<Entity<'a>>>) -> Entity<'a> {
    self.alloc(CollectedEntity { val, deps: collected, consumed: Cell::new(false) })
  }
}
