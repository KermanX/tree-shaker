use super::{
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
};
use crate::analyzer::Analyzer;
use rustc_hash::FxHashSet;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct ForwardedEntity<'a> {
  val: Entity<'a>,
  dep: EntityDep<'a>,
}

impl<'a> EntityTrait<'a> for ForwardedEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {
    analyzer.refer_dep(&self.dep);
    self.val.consume_self(analyzer)
  }

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    analyzer.refer_dep(&self.dep);
    self.val.consume_as_unknown(analyzer)
  }

  fn get_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> (bool, Entity<'a>) {
    let (has_effect, value) = self.val.get_property(analyzer, key);
    (has_effect, self.forward(value))
  }

  fn set_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool {
    self.val.set_property(analyzer, key, value)
  }

  fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
    let (has_effect, properties) = self.val.enumerate_properties(analyzer);
    (
      has_effect,
      properties
        .into_iter()
        .map(|(definite, key, value)| (definite, key, self.forward(value)))
        .collect(),
    )
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
    self.val.delete_property(analyzer, key)
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    let (has_effect, ret_val) = self.val.call(analyzer, this, args);
    if has_effect {
      self.consume_self(analyzer);
    }
    (has_effect, self.forward(ret_val))
  }

  fn r#await(&self, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    let (has_effect, ret_val) = self.val.r#await(analyzer);
    if has_effect {
      self.consume_self(analyzer);
    }
    (has_effect, self.forward(ret_val))
  }

  fn get_typeof(&self) -> Entity<'a> {
    self.forward(self.val.get_typeof())
  }

  fn get_to_string(&self) -> Entity<'a> {
    self.forward(self.val.get_to_string())
  }

  fn get_to_property_key(&self) -> Entity<'a> {
    self.forward(self.val.get_to_property_key())
  }

  fn get_to_array(&self, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    let (items, rest) = self.val.get_to_array(length);
    (items.into_iter().map(|item| self.forward(item)).collect(), self.forward(rest))
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

impl<'a> ForwardedEntity<'a> {
  pub(crate) fn new(val: Entity<'a>, dep: EntityDep<'a>) -> Entity<'a> {
    Rc::new(Self { val, dep })
  }

  pub(crate) fn forward(&self, val: Entity<'a>) -> Entity<'a> {
    ForwardedEntity::new(val, self.dep.clone())
  }
}
