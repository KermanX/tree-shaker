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
    (has_effect, ForwardedEntity::new(value, self.dep.clone()))
  }

  fn set_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool {
    self.val.set_property(analyzer, key, value)
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    self.val.call(analyzer, this, args)
  }

  fn get_typeof(&self) -> Entity<'a> {
    ForwardedEntity::new(self.val.get_typeof(), self.dep.clone())
  }

  fn get_to_string(&self) -> Entity<'a> {
    ForwardedEntity::new(self.val.get_to_string(), self.dep.clone())
  }

  fn get_to_property_key(&self) -> Entity<'a> {
    ForwardedEntity::new(self.val.get_to_property_key(), self.dep.clone())
  }

  fn get_to_array(&self, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    let (items, rest) = self.val.get_to_array(length);
    (
      items.into_iter().map(|item| ForwardedEntity::new(item, self.dep.clone())).collect(),
      ForwardedEntity::new(rest, self.dep.clone()),
    )
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
}
