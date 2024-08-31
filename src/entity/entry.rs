use super::{
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
};
use crate::analyzer::Analyzer;
use rustc_hash::FxHashSet;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct EntryEntity<'a> {
  pub key: Entity<'a>,
  pub value: Entity<'a>,
}

impl<'a> EntityTrait<'a> for EntryEntity<'a> {
  fn consume_self(&self, analyzer: &mut crate::analyzer::Analyzer<'a>) {
    self.key.consume_self(analyzer);
    self.value.consume_self(analyzer);
  }

  fn consume_as_unknown(&self, analyzer: &mut crate::analyzer::Analyzer<'a>) {
    self.key.consume_self(analyzer);
    self.value.consume_as_unknown(analyzer);
  }

  fn get_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> (bool, Entity<'a>) {
    let (has_effect, value) = self.value.get_property(analyzer, key);
    (has_effect, self.forward(value))
  }

  fn set_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool {
    self.value.set_property(analyzer, key, value)
  }

  fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(Entity<'a>, Entity<'a>)>) {
    let (has_effect, properties) = self.value.enumerate_properties(analyzer);
    (has_effect, properties.into_iter().map(|(key, value)| (key, self.forward(value))).collect())
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    let (has_effect, ret_val) = self.value.call(analyzer, this, args);
    (has_effect, self.forward(ret_val))
  }

  fn get_typeof(&self) -> Entity<'a> {
    self.forward(self.value.get_typeof())
  }

  fn get_to_string(&self) -> Entity<'a> {
    self.forward(self.value.get_to_string())
  }

  fn get_to_property_key(&self) -> Entity<'a> {
    self.forward(self.value.get_to_property_key())
  }

  fn get_to_array(&self, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    let (vals, ret_val) = self.value.get_to_array(length);
    (vals.iter().map(|val| self.forward(val.clone())).collect(), self.forward(ret_val))
  }

  fn get_to_literals(&self) -> Option<FxHashSet<LiteralEntity<'a>>> {
    self.value.get_to_literals()
  }

  fn test_typeof(&self) -> TypeofResult {
    self.value.test_typeof()
  }

  fn test_truthy(&self) -> Option<bool> {
    self.value.test_truthy()
  }

  fn test_nullish(&self) -> Option<bool> {
    self.value.test_nullish()
  }
}

impl<'a> EntryEntity<'a> {
  pub fn new(value: Entity<'a>, key: Entity<'a>) -> Entity<'a> {
    Rc::new(EntryEntity { key, value })
  }

  fn forward(&self, value: Entity<'a>) -> Entity<'a> {
    EntryEntity::new(value, self.key.clone())
  }
}
