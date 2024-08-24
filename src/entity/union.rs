use super::{
  entity::{Entity, EntityTrait},
  unknown::UnknownEntity,
};
use crate::analyzer::Analyzer;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct UnionEntity<'a>(pub Vec<Entity<'a>>);

impl<'a> EntityTrait<'a> for UnionEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {
    for entity in &self.0 {
      entity.consume_self(analyzer);
    }
  }

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    for entity in &self.0 {
      entity.consume_as_unknown(analyzer);
    }
  }

  fn consume_as_array(
    &self,
    analyzer: &mut Analyzer<'a>,
    length: usize,
  ) -> (Vec<Entity<'a>>, Entity<'a>) {
    // FIXME: May have the same result
    for entity in &self.0 {
      entity.consume_as_array(analyzer, length);
    }
    let mut result = Vec::new();
    for _ in 0..length {
      result.push(UnknownEntity::new_unknown());
    }
    (result, UnknownEntity::new_unknown())
  }

  fn get_property(&self, key: &Entity<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    for entity in &self.0 {
      result.push(entity.get_property(key));
    }
    Rc::new(UnionEntity(result))
  }

  fn test_truthy(&self) -> Option<bool> {
    let result = self.0.first().unwrap().test_truthy()?;
    for entity in &self.0[1..] {
      if entity.test_truthy()? != result {
        return None;
      }
    }
    Some(result)
  }
}

impl<'a> UnionEntity<'a> {
  pub(crate) fn new(entities: Vec<Entity<'a>>) -> Entity<'a> {
    Rc::new(UnionEntity(entities))
  }
}
