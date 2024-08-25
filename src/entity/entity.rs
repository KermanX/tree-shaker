use super::{literal::LiteralEntity, unknown::UnknownEntity};
use crate::analyzer::Analyzer;
use std::{fmt::Debug, rc::Rc};

pub(crate) trait EntityTrait<'a>: Debug {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>);
  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    self.consume_self(analyzer);
  }
  fn consume_as_array(
    &self,
    _analyzer: &mut Analyzer<'a>,
    length: usize,
  ) -> (Vec<Entity<'a>>, Entity<'a>) {
    let mut result = Vec::new();
    for _ in 0..length {
      result.push(UnknownEntity::new_unknown());
    }
    (result, UnknownEntity::new_unknown())
  }
  fn call(
    &self,
    _analyzer: &mut Analyzer<'a>,
    _this: &Entity<'a>,
    _args: &Entity<'a>,
  ) -> Entity<'a> {
    UnknownEntity::new_unknown()
  }
  fn get_property(&self, key: &Entity<'a>) -> Entity<'a>;
  fn get_literal(&self) -> Option<LiteralEntity<'a>> {
    None
  }
  fn test_truthy(&self) -> Option<bool> {
    None
  }
  fn test_nullish(&self) -> Option<bool> {
    None
  }
}

pub(crate) type Entity<'a> = Rc<dyn EntityTrait<'a> + 'a>;

impl<'a> Analyzer<'a> {
  pub(crate) fn consume_entity(&mut self, entity: &Entity<'a>) {
    entity.consume_self(self);
  }

  pub(crate) fn consume_as_array(
    &mut self,
    entity: &Entity<'a>,
    length: usize,
  ) -> (Vec<Entity<'a>>, Entity<'a>) {
    entity.consume_as_array(self, length)
  }
}
