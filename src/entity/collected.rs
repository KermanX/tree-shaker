use super::{
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
};
use crate::analyzer::Analyzer;
use rustc_hash::FxHashSet;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct CollectedEntity<'a> {
  val: Entity<'a>,
  collected: Rc<RefCell<Vec<Entity<'a>>>>,
}

impl<'a> EntityTrait<'a> for CollectedEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    for entity in self.collected.borrow().iter() {
      entity.consume(analyzer);
    }
    self.val.consume(analyzer)
  }

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    let value = self.val.get_property(analyzer, dep, key);
    self.forward(value)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    for entity in self.collected.borrow().iter() {
      entity.consume(analyzer);
    }
    self.val.set_property(analyzer, dep, key, value)
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    for entity in self.collected.borrow().iter() {
      entity.consume(analyzer);
    }
    self.val.enumerate_properties(analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep, key: &Entity<'a>) {
    for entity in self.collected.borrow().iter() {
      entity.consume(analyzer);
    }
    self.val.delete_property(analyzer, dep, key)
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    let ret_cal = self.val.call(analyzer, dep, this, args);
    self.forward(ret_cal)
  }

  fn r#await(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    let (has_effect, ret_val) = self.val.r#await(analyzer);
    if has_effect {
      self.consume(analyzer);
    }
    (has_effect, self.forward(ret_val))
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    let (elements, rest) = self.val.iterate(analyzer, dep);
    (elements.into_iter().map(|v| self.forward(v)).collect(), rest.map(|v| self.forward(v)))
  }

  fn get_typeof(&self) -> Entity<'a> {
    // TODO: Verify this
    self.forward(self.val.get_typeof())
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_string())
  }

  fn get_to_numeric(&self, _rc: &Entity<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_numeric())
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_property_key())
  }

  fn get_to_literals(&self) -> Option<FxHashSet<LiteralEntity<'a>>> {
    self.val.get_to_literals()
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
  pub fn new(val: Entity<'a>, collected: Rc<RefCell<Vec<Entity<'a>>>>) -> Entity<'a> {
    Entity::new(Self { val, collected })
  }

  fn forward(&self, val: Entity<'a>) -> Entity<'a> {
    CollectedEntity::new(val, self.collected.clone())
  }
}
