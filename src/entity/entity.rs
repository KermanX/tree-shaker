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
  ) -> (bool, Entity<'a>) {
    (true, UnknownEntity::new_unknown())
  }
  fn get_typeof(&self) -> Entity<'a>;
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
  fn test_is_undefined(&self) -> Option<bool> {
    None
  }
}

pub(crate) type Entity<'a> = Rc<dyn EntityTrait<'a> + 'a>;
