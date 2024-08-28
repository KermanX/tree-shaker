use super::{
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
};
use crate::analyzer::Analyzer;
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

  fn consume_as_array(
    &self,
    analyzer: &mut Analyzer<'a>,
    length: usize,
  ) -> (Vec<Entity<'a>>, Entity<'a>) {
    for entity in self.collected.borrow().iter() {
      entity.consume_as_array(analyzer, length);
    }
    self.val.consume_as_array(analyzer, length)
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

  fn get_property(&self, key: &Entity<'a>) -> Entity<'a> {
    CollectedEntity::new(self.val.get_property(key), self.collected.clone())
  }

  fn get_literal(&self) -> Option<LiteralEntity<'a>> {
    self.val.get_literal()
  }

  fn test_truthy(&self) -> Option<bool> {
    self.val.test_truthy()
  }

  fn test_nullish(&self) -> Option<bool> {
    self.val.test_nullish()
  }

  fn test_is_undefined(&self) -> Option<bool> {
    self.val.test_is_undefined()
  }
}

impl<'a> CollectedEntity<'a> {
  pub(crate) fn new(val: Entity<'a>, collected: Rc<RefCell<Vec<Entity<'a>>>>) -> Entity<'a> {
    Rc::new(Self { val, collected })
  }
}
