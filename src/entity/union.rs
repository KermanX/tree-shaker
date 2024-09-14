use super::{
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::UnknownEntity,
  utils::collect_effect_and_value,
};
use crate::{analyzer::Analyzer, scope::CfScopeKind};
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct UnionEntity<'a>(pub Vec<Entity<'a>>);

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

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    key: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    let mut values = Vec::new();
    for entity in &self.0 {
      values.push(entity.get_property(analyzer, key));
    }
    collect_effect_and_value(values)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) -> bool {
    let mut has_effect = false;
    for entity in &self.0 {
      has_effect |= entity.set_property(analyzer, key, value.clone());
    }
    has_effect
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
    // FIXME:
    UnknownEntity::new_unknown_to_entries_result(self.0.clone())
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
    let mut deleted = false;
    for entity in &self.0 {
      deleted |= entity.delete_property(analyzer, key);
    }
    deleted
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    let mut results = Vec::new();
    analyzer.push_cf_scope(CfScopeKind::Normal, None, None);
    for entity in &self.0 {
      results.push(entity.call(analyzer, this, args));
    }
    analyzer.pop_cf_scope();
    collect_effect_and_value(results)
  }

  fn r#await(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    let mut results = Vec::new();
    for entity in &self.0 {
      results.push(entity.r#await(analyzer));
    }
    collect_effect_and_value(results)
  }

  fn iterate(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Option<Entity<'a>>) {
    let mut has_effect = false;
    let mut results = vec![];
    for entity in &self.0 {
      let (effect, result) = entity.iterate(analyzer);
      has_effect |= effect;
      if let Some(result) = result {
        results.push(result);
      }
    }
    (has_effect, if results.is_empty() { None } else { Some(UnionEntity::new(results)) })
  }

  fn get_typeof(&self) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: collect literals
    for entity in &self.0 {
      result.push(entity.get_typeof());
    }
    UnionEntity::new(result)
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: dedupe
    for entity in &self.0 {
      result.push(entity.get_to_string());
    }
    UnionEntity::new(result)
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: dedupe
    for entity in &self.0 {
      result.push(entity.get_to_property_key());
    }
    UnionEntity::new(result)
  }

  fn get_to_array(&self, _rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    // FIXME: May have the same result
    let mut elements = Vec::new();
    for _ in 0..length {
      elements.push(Vec::new());
    }
    let mut rest = Vec::new();
    for entity in &self.0 {
      let result = entity.get_to_array(length);
      for (i, element) in elements.iter_mut().enumerate() {
        element.push(result.0[i].clone());
      }
      rest.push(result.1);
    }
    (elements.into_iter().map(UnionEntity::new).collect(), UnionEntity::new(rest))
  }

  fn get_to_literals(&self) -> Option<FxHashSet<LiteralEntity<'a>>> {
    let mut result = self.0.first().unwrap().get_to_literals()?;
    for entity in &self.0[1..] {
      result.extend(entity.get_to_literals()?);
    }
    Some(result)
  }

  fn test_typeof(&self) -> TypeofResult {
    let mut result = TypeofResult::_None;
    for entity in &self.0 {
      result |= entity.test_typeof();
    }
    result
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

  fn test_nullish(&self) -> Option<bool> {
    let result = self.0.first().unwrap().test_nullish()?;
    for entity in &self.0[1..] {
      if entity.test_nullish()? != result {
        return None;
      }
    }
    Some(result)
  }
}

impl<'a> UnionEntity<'a> {
  pub fn new(entities: Vec<Entity<'a>>) -> Entity<'a> {
    debug_assert!(!entities.is_empty());
    if entities.len() == 1 {
      entities.first().unwrap().clone()
    } else {
      let has_unknown = entities.iter().any(|entity| entity.test_is_completely_unknown());
      if has_unknown {
        UnknownEntity::new_unknown()
      } else {
        Entity::new(UnionEntity(entities))
      }
    }
  }
}
