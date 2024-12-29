use super::{
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{analyzer::Analyzer, consumable::Consumable, use_consumed_flag};
use rustc_hash::FxHashSet;
use std::{
  cell::{Cell, RefCell},
  rc::Rc,
};

#[derive(Debug)]
pub struct CollectedEntity<'a> {
  val: Entity<'a>,
  deps: Rc<RefCell<Vec<Entity<'a>>>>,
  consumed: Cell<bool>,
}

impl<'a> EntityTrait<'a> for CollectedEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);
    self.consume_deps(analyzer);
    self.val.consume(analyzer);
  }

  fn consume_mangable(&self, analyzer: &mut Analyzer<'a>) -> bool {
    self.consume_deps(analyzer);
    self.val.consume_mangable(analyzer)
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    self.val.unknown_mutate(analyzer, dep)
  }

  fn get_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    let value = self.val.get_property(analyzer, dep, key);
    self.forward(value, analyzer)
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.consume_deps(analyzer);
    self.val.set_property(analyzer, dep, key, value)
  }

  fn enumerate_properties(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    self.consume_deps(analyzer);
    self.val.enumerate_properties(analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.consume_deps(analyzer);
    self.val.delete_property(analyzer, dep, key)
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let value = self.val.call(analyzer, dep, this, args);
    self.forward(value, analyzer)
  }

  fn construct(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let value = self.val.construct(analyzer, dep, args);
    self.forward(value, analyzer)
  }

  fn jsx(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    analyzer.factory.computed(self.val.jsx(analyzer, props), self.deps.clone())
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    self.forward(self.val.r#await(analyzer, dep), analyzer)
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    let (elements, rest, deps) = self.val.iterate(analyzer, dep);
    (elements, rest, analyzer.consumable((deps, self.deps.clone())))
  }

  fn get_destructable(
    &self,
    _rc: Entity<'a>,
    analyzer: &Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Consumable<'a> {
    analyzer.consumable((self.deps.clone(), dep))
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    // TODO: Verify this
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

  fn get_to_jsx_child(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_jsx_child(analyzer), analyzer)
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

impl<'a> CollectedEntity<'a> {
  fn forward(&self, val: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if self.consumed.get() || self.deps.borrow().is_empty() {
      val
    } else {
      analyzer.factory.collected(val, self.deps.clone())
    }
  }

  fn consume_deps(&self, analyzer: &mut Analyzer<'a>) {
    for entity in self.deps.take().iter() {
      entity.consume_mangable(analyzer);
    }
  }
}

impl<'a> EntityFactory<'a> {
  pub fn collected(
    &self,
    val: Entity<'a>,
    collected: impl Into<Rc<RefCell<Vec<Entity<'a>>>>>,
  ) -> Entity<'a> {
    self.entity(CollectedEntity { val, deps: collected.into(), consumed: Cell::new(false) })
  }
}
