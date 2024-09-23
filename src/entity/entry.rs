use super::{
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  interactions::InteractionKind,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
};
use crate::analyzer::Analyzer;
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct EntryEntity<'a> {
  pub key: Entity<'a>,
  pub value: Entity<'a>,
}

impl<'a> EntityTrait<'a> for EntryEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.key.consume(analyzer);
    self.value.consume(analyzer);
  }

  fn interact(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep, kind: InteractionKind) {
    self.key.interact(analyzer, dep.clone(), kind);
    self.value.interact(analyzer, dep, kind);
  }

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    let value = self.value.get_property(analyzer, dep, key);
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
    self.value.set_property(analyzer, dep, key, value);
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    self
      .value
      .enumerate_properties(analyzer, dep)
      .into_iter()
      .map(|(definite, key, value)| (definite, key, self.forward(value)))
      .collect()
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep, key: &Entity<'a>) {
    self.value.delete_property(analyzer, dep, key)
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    let ret_val = self.value.call(analyzer, dep, this, args);
    self.forward(ret_val)
  }

  fn r#await(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.forward(self.value.r#await(analyzer))
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    let (elements, rest) = self.value.iterate(analyzer, dep);
    (elements.into_iter().map(|v| self.forward(v)).collect(), rest.map(|v| self.forward(v)))
  }

  fn get_typeof(&self) -> Entity<'a> {
    self.forward(self.value.get_typeof())
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    self.forward(self.value.get_to_string())
  }

  fn get_to_numeric(&self, _rc: &Entity<'a>) -> Entity<'a> {
    self.forward(self.value.get_to_numeric())
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    self.forward(self.value.get_to_property_key())
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
    Entity::new(EntryEntity { key, value })
  }

  fn forward(&self, value: Entity<'a>) -> Entity<'a> {
    EntryEntity::new(value, self.key.clone())
  }
}
