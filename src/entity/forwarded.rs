use super::{
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
};
use crate::analyzer::Analyzer;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct ForwardedEntity<'a> {
  val: Entity<'a>,
  deps: Vec<EntityDep<'a>>,
}

impl<'a> EntityTrait<'a> for ForwardedEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {
    self.refer_deps(analyzer);
    self.val.consume_self(analyzer)
  }

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    self.refer_deps(analyzer);
    self.val.consume_as_unknown(analyzer)
  }

  fn consume_as_array(
    &self,
    analyzer: &mut Analyzer<'a>,
    length: usize,
  ) -> (Vec<Entity<'a>>, Entity<'a>) {
    self.refer_deps(analyzer);
    self.val.consume_as_array(analyzer, length)
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    self.refer_deps(analyzer);
    self.val.call(analyzer, this, args)
  }

  fn test_truthy(&self) -> Option<bool> {
    self.val.test_truthy()
  }

  fn test_nullish(&self) -> Option<bool> {
    self.val.test_nullish()
  }

  fn get_literal(&self) -> Option<LiteralEntity<'a>> {
    self.val.get_literal()
  }

  fn get_property(&self, key: &Entity<'a>) -> Entity<'a> {
    self.val.get_property(key)
  }
}

impl<'a> ForwardedEntity<'a> {
  pub(crate) fn new(val: Entity<'a>, deps: Vec<EntityDep<'a>>) -> Entity<'a> {
    Rc::new(Self { val, deps })
  }

  fn refer_deps(&self, analyzer: &mut Analyzer<'a>) {
    for dep in &self.deps {
      analyzer.refer_dep(dep);
    }
  }
}
