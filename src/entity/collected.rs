use super::{
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
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {
    for entity in self.collected.borrow().iter() {
      entity.consume_self(analyzer);
    }
    self.val.consume_self(analyzer)
  }

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    for entity in self.collected.borrow().iter() {
      entity.consume_as_unknown(analyzer);
    }
    self.val.consume_as_unknown(analyzer)
  }

  fn get_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> (bool, Entity<'a>) {
    let (has_effect, value) = self.val.get_property(analyzer, key);
    (has_effect, self.forward(value))
  }

  fn set_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool {
    for entity in self.collected.borrow().iter() {
      entity.consume_self(analyzer);
    }
    self.val.set_property(analyzer, key, value)
  }

  fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
    for entity in self.collected.borrow().iter() {
      entity.consume_self(analyzer);
    }
    self.val.enumerate_properties(analyzer)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
    for entity in self.collected.borrow().iter() {
      entity.consume_self(analyzer);
    }
    self.val.delete_property(analyzer, key)
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    let (has_effect, ret_cal) = self.val.call(analyzer, this, args);
    if has_effect {
      self.consume_self(analyzer);
    }
    (has_effect, self.forward(ret_cal))
  }

  fn r#await(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    let (has_effect, ret_val) = self.val.r#await(analyzer);
    if has_effect {
      self.consume_self(analyzer);
    }
    (has_effect, self.forward(ret_val))
  }

  fn get_typeof(&self) -> Entity<'a> {
    // TODO: Verify this
    self.forward(self.val.get_typeof())
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_string())
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    self.forward(self.val.get_to_property_key())
  }

  fn get_to_array(&self, _rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    let (elements, rest) = self.val.get_to_array(length);
    (elements.into_iter().map(|entity| self.forward(entity)).collect(), self.forward(rest))
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
