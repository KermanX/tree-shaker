use super::{
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
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
    let mut elements = Vec::new();
    for _ in 0..length {
      elements.push(Vec::new());
    }
    let mut rest = Vec::new();
    for entity in &self.0 {
      let result = entity.consume_as_array(analyzer, length);
      for (i, element) in elements.iter_mut().enumerate() {
        element.push(result.0[i].clone());
      }
      rest.push(result.1);
    }
    (elements.into_iter().map(UnionEntity::new).collect(), UnionEntity::new(rest))
  }

  fn call(&self, analyzer: &mut Analyzer<'a>, this: &Entity<'a>, args: &Entity<'a>) -> Entity<'a> {
    let mut ret_val = Vec::new();
    for entity in &self.0 {
      ret_val.push(entity.call(analyzer, this, args));
    }
    UnionEntity::new(ret_val)
  }

  fn get_property(&self, key: &Entity<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    for entity in &self.0 {
      result.push(entity.get_property(key));
    }
    Rc::new(UnionEntity(result))
  }

  fn get_literal(&self) -> Option<LiteralEntity<'a>> {
    let result = self.0.first().unwrap().get_literal()?;
    for entity in &self.0[1..] {
      if entity.get_literal()? != result {
        return None;
      }
    }
    Some(result)
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
