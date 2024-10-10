use super::{
  consumed_object, ComputedEntity, Entity, EntityTrait, LiteralEntity, TypeofResult, UnknownEntity,
};
use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableNode, ConsumableTrait},
  use_consumed_flag,
};
use rustc_hash::FxHashSet;
use std::cell::Cell;

#[derive(Debug)]
pub struct UnionEntity<'a> {
  /// Possible values
  pub values: Vec<Entity<'a>>,
  consumed: Cell<bool>,
}

impl<'a> EntityTrait<'a> for UnionEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    for value in &self.values {
      value.consume(analyzer);
    }
  }

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    let mut values = Vec::new();
    for entity in &self.values {
      values.push(
        analyzer.exec_indeterminately(|analyzer| entity.get_property(analyzer, dep.cloned(), key)),
      );
    }
    UnionEntity::new(values)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    for entity in &self.values {
      analyzer.exec_indeterminately(|analyzer| {
        entity.set_property(analyzer, dep.cloned(), key, value.clone())
      });
    }
  }

  fn enumerate_properties(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    // FIXME:
    if analyzer.config.unknown_property_read_side_effects {
      self.consume(analyzer);
    }
    consumed_object::enumerate_properties(rc, analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: &Entity<'a>) {
    for entity in &self.values {
      analyzer.exec_indeterminately(|analyzer| entity.delete_property(analyzer, dep.cloned(), key));
    }
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    let mut results = Vec::new();
    for entity in &self.values {
      results.push(
        analyzer.exec_indeterminately(|analyzer| entity.call(analyzer, dep.cloned(), this, args)),
      );
    }
    UnionEntity::new(results)
  }

  fn r#await(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    let mut values = Vec::new();
    for entity in &self.values {
      values.push(analyzer.exec_indeterminately(|analyzer| entity.r#await(analyzer, dep.cloned())));
    }
    UnionEntity::new(values)
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    let mut results = Vec::new();
    let mut has_undefined = false;
    for entity in &self.values {
      if let Some(result) = analyzer
        .exec_indeterminately(|analyzer| entity.iterate_result_union(analyzer, dep.cloned()))
      {
        results.push(result);
      } else {
        has_undefined = true;
      }
    }
    if has_undefined {
      results.push(LiteralEntity::new_undefined());
    }
    (vec![], UnionEntity::try_new(results))
  }

  fn get_typeof(&self) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: collect literals
    for entity in &self.values {
      result.push(entity.get_typeof());
    }
    UnionEntity::new(result)
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: dedupe
    for entity in &self.values {
      result.push(entity.get_to_string());
    }
    UnionEntity::new(result)
  }

  fn get_to_numeric(&self, _rc: &Entity<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: dedupe
    for entity in &self.values {
      result.push(entity.get_to_numeric());
    }
    UnionEntity::new(result)
  }

  fn get_to_boolean(&self, _rc: &Entity<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    for entity in &self.values {
      result.push(entity.get_to_boolean());
    }
    UnionEntity::new(result)
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: dedupe
    for entity in &self.values {
      result.push(entity.get_to_property_key());
    }
    UnionEntity::new(result)
  }

  fn get_to_literals(&self) -> Option<FxHashSet<LiteralEntity<'a>>> {
    let mut result = self.values.first().unwrap().get_to_literals()?;
    for entity in &self.values[1..] {
      result.extend(entity.get_to_literals()?);
    }
    Some(result)
  }

  fn test_typeof(&self) -> TypeofResult {
    let mut result = TypeofResult::_None;
    for entity in &self.values {
      result |= entity.test_typeof();
    }
    result
  }

  fn test_truthy(&self) -> Option<bool> {
    let result = self.values.first().unwrap().test_truthy()?;
    for entity in &self.values[1..] {
      if entity.test_truthy()? != result {
        return None;
      }
    }
    Some(result)
  }

  fn test_nullish(&self) -> Option<bool> {
    let result = self.values.first().unwrap().test_nullish()?;
    for entity in &self.values[1..] {
      if entity.test_nullish()? != result {
        return None;
      }
    }
    Some(result)
  }
}

impl<'a> UnionEntity<'a> {
  pub fn try_new(values: Vec<Entity<'a>>) -> Option<Entity<'a>> {
    if values.is_empty() {
      None
    } else {
      Some(if values.len() == 1 {
        values.first().unwrap().clone()
      } else {
        let has_unknown = values.iter().any(|entity| entity.test_is_completely_unknown());
        if has_unknown {
          UnknownEntity::new_computed_unknown(ConsumableNode::new_box(values).into())
        } else {
          Entity::new(UnionEntity { values, consumed: Cell::new(false) })
        }
      })
    }
  }

  pub fn new(values: Vec<Entity<'a>>) -> Entity<'a> {
    Self::try_new(values).unwrap()
  }

  pub fn new_computed(values: Vec<Entity<'a>>, dep: Consumable<'a>) -> Entity<'a> {
    ComputedEntity::new(Self::new(values), dep)
  }
}
