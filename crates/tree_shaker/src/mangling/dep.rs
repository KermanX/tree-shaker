use crate::{
  analyzer::Analyzer,
  consumable::ConsumableTrait,
  entity::{Entity, EntityFactory},
  mangling::MangleConstraint,
};

#[derive(Debug, Clone, Copy)]
pub struct ManglingDep<'a> {
  deps: (Entity<'a>, Entity<'a>),
  constraint: &'a MangleConstraint,
}

impl<'a> ConsumableTrait<'a> for ManglingDep<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.deps.0.consume_mangable(analyzer);
    self.deps.1.consume_mangable(analyzer);
    analyzer.consume(self.constraint);
  }
}

impl<'a> EntityFactory<'a> {
  pub fn mangable(
    &self,
    val: Entity<'a>,
    deps: (Entity<'a>, Entity<'a>),
    constraint: &'a MangleConstraint,
  ) -> Entity<'a> {
    self.computed(val, ManglingDep { deps, constraint })
  }
}
