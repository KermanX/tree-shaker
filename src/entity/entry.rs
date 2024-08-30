use super::{
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
};
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

  fn get_property(&self, key: &Entity<'a>) -> Entity<'a> {
    EntryEntity::new(self.value.get_property(key), self.key.clone())
  }

  fn set_property(&self, key: &Entity<'a>, value: Entity<'a>) {
    self.value.set_property(key, value);
  }

  fn call(
    &self,
    analyzer: &mut crate::analyzer::Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    let (has_effect, ret_val) = self.value.call(analyzer, this, args);
    (has_effect, EntryEntity::new(ret_val, self.key.clone()))
  }

  fn get_typeof(&self) -> Entity<'a> {
    EntryEntity::new(self.value.get_typeof(), self.key.clone())
  }

  fn get_to_string(&self) -> Entity<'a> {
    EntryEntity::new(self.value.get_to_string(), self.key.clone())
  }

  fn get_to_property_key(&self) -> Entity<'a> {
    EntryEntity::new(self.value.get_to_property_key(), self.key.clone())
  }

  fn get_to_array(&self, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    let (vals, ret_val) = self.value.get_to_array(length);
    (
      vals.iter().map(|val| EntryEntity::new(val.clone(), self.key.clone())).collect(),
      EntryEntity::new(ret_val, self.key.clone()),
    )
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
}
