use super::{
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
};
use crate::analyzer::Analyzer;
use rustc_hash::FxHashSet;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub(crate) struct CollectedEntity<'a> {
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

  fn get_property(&self, key: &Entity<'a>) -> (bool, Entity<'a>) {
    let (has_effect, value) = self.val.get_property(key);
    (has_effect, CollectedEntity::new(value, self.collected.clone()))
  }

  fn set_property(&self, key: &Entity<'a>, value: Entity<'a>) -> bool {
    // self.collected are all literals, setting their properties has no effect
    self.val.set_property(key, value)
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    for entity in self.collected.borrow().iter() {
      entity.call(analyzer, this, args);
    }
    self.val.call(analyzer, this, args)
  }

  fn get_typeof(&self) -> Entity<'a> {
    // TODO: Verify this
    CollectedEntity::new(self.val.get_typeof(), self.collected.clone())
  }

  fn get_to_string(&self) -> Entity<'a> {
    CollectedEntity::new(self.val.get_to_string(), self.collected.clone())
  }

  fn get_to_property_key(&self) -> Entity<'a> {
    CollectedEntity::new(self.val.get_to_property_key(), self.collected.clone())
  }

  fn get_to_array(&self, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    for entity in self.collected.borrow().iter() {
      entity.get_to_array(length);
    }
    self.val.get_to_array(length)
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
  pub(crate) fn new(val: Entity<'a>, collected: Rc<RefCell<Vec<Entity<'a>>>>) -> Entity<'a> {
    Rc::new(Self { val, collected })
  }
}
