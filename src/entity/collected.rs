use super::{Entity, EntityFactory, EntityTrait, LiteralEntity, TypeofResult};
use crate::{analyzer::Analyzer, consumable::Consumable};
use rustc_hash::FxHashSet;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct CollectedEntity<'a> {
  val: Entity<'a>,
  deps: Rc<RefCell<Vec<Entity<'a>>>>,
}

impl<'a> EntityTrait<'a> for CollectedEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    for entity in self.deps.borrow().iter() {
      entity.consume(analyzer);
    }
    self.val.consume(analyzer)
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
    for entity in self.deps.borrow().iter() {
      entity.consume(analyzer);
    }
    self.val.set_property(analyzer, dep, key, value)
  }

  fn enumerate_properties(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    for entity in self.deps.borrow().iter() {
      entity.consume(analyzer);
    }
    self.val.enumerate_properties(analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    for entity in self.deps.borrow().iter() {
      entity.consume(analyzer);
    }
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
    let ret_cal = self.val.call(analyzer, dep, this, args);
    self.forward(ret_cal, analyzer)
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
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    let (elements, rest) = self.val.iterate(analyzer, dep);
    (
      elements.into_iter().map(|v| self.forward(v, analyzer)).collect(),
      rest.map(|v| self.forward(v, analyzer)),
    )
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
    analyzer.factory.new_collected(val, self.deps.clone())
  }
}

impl<'a> EntityFactory<'a> {
  pub fn new_collected(
    &self,
    val: Entity<'a>,
    collected: impl Into<Rc<RefCell<Vec<Entity<'a>>>>>,
  ) -> Entity<'a> {
    self.new_entity(CollectedEntity { val, deps: collected.into() })
  }
}
