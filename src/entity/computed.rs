use super::{Consumable, Entity, EntityTrait, InteractionKind, LiteralEntity, TypeofResult};
use crate::analyzer::Analyzer;
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct ComputedEntity<'a> {
  val: Entity<'a>,
  dep: Consumable<'a>,
}

impl<'a> EntityTrait<'a> for ComputedEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.val.consume(analyzer);
    self.dep.consume(analyzer);
  }

  fn interact(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, kind: InteractionKind) {
    self.val.interact(analyzer, (self.dep.clone(), dep), kind);
  }

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    self.val.get_property(analyzer, (self.dep.clone(), dep), key)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    self.val.set_property(analyzer, (self.dep.clone(), dep), key, value);
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    self.val.enumerate_properties(analyzer, (self.dep.clone(), dep))
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: &Entity<'a>) {
    self.val.delete_property(analyzer, (self.dep.clone(), dep), key)
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.val.call(analyzer, (self.dep.clone(), dep), this, args)
  }

  fn r#await(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    self.val.r#await(analyzer, (self.dep.clone(), dep))
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    self.val.iterate(analyzer, (self.dep.clone(), dep))
  }

  fn get_typeof(&self) -> Entity<'a> {
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

impl<'a> ComputedEntity<'a> {
  pub fn new(val: Entity<'a>, dep: impl Into<Consumable<'a>>) -> Entity<'a> {
    Entity::new(Self { val, dep: dep.into() })
  }

  pub fn forward(&self, val: Entity<'a>) -> Entity<'a> {
    ComputedEntity::new(val, self.dep.clone())
  }
}
