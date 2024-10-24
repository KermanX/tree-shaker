use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable, ConsumableTrait},
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
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    let mut values = Vec::new();
    for entity in &self.values {
      values.push(
        analyzer.exec_indeterminately(|analyzer| entity.get_property(analyzer, dep.cloned(), key)),
      );
    }
    analyzer.factory.union(values)
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
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
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    // FIXME:
    if analyzer.config.unknown_property_read_side_effects {
      self.consume(analyzer);
    }
    consumed_object::enumerate_properties(rc, analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    for entity in &self.values {
      analyzer.exec_indeterminately(|analyzer| entity.delete_property(analyzer, dep.cloned(), key));
    }
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let mut results = Vec::new();
    for entity in &self.values {
      results.push(
        analyzer.exec_indeterminately(|analyzer| entity.call(analyzer, dep.cloned(), this, args)),
      );
    }
    analyzer.factory.union(results)
  }

  fn construct(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let mut results = Vec::new();
    for entity in &self.values {
      results.push(
        analyzer
          .exec_indeterminately(|analyzer| entity.construct(analyzer, dep.cloned(), args.clone())),
      );
    }
    analyzer.factory.union(results)
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    let mut values = Vec::new();
    for entity in &self.values {
      values.push(analyzer.exec_indeterminately(|analyzer| entity.r#await(analyzer, dep.cloned())));
    }
    analyzer.factory.union(values)
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
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
      results.push(analyzer.factory.undefined);
    }
    (vec![], analyzer.factory.try_union(results), box_consumable(()))
  }

  fn get_destructable(&self, _rc: Entity<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    let mut values = Vec::new();
    for entity in &self.values {
      values.push(entity.get_destructable(dep.cloned()));
    }
    box_consumable(values)
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: collect literals
    for entity in &self.values {
      result.push(entity.get_typeof(analyzer));
    }
    analyzer.factory.union(result)
  }

  fn get_to_string(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: dedupe
    for entity in &self.values {
      result.push(entity.get_to_string(analyzer));
    }
    analyzer.factory.union(result)
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: dedupe
    for entity in &self.values {
      result.push(entity.get_to_numeric(analyzer));
    }
    analyzer.factory.union(result)
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    for entity in &self.values {
      result.push(entity.get_to_boolean(analyzer));
    }
    analyzer.factory.union(result)
  }

  fn get_to_property_key(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    let mut result = Vec::new();
    // TODO: dedupe
    for entity in &self.values {
      result.push(entity.get_to_property_key(analyzer));
    }
    analyzer.factory.union(result)
  }

  fn get_to_literals(
    &self,
    _rc: Entity<'a>,
    analyzer: &Analyzer<'a>,
  ) -> Option<FxHashSet<LiteralEntity<'a>>> {
    let mut result = self.values.first().unwrap().get_to_literals(analyzer)?;
    for entity in &self.values[1..] {
      result.extend(entity.get_to_literals(analyzer)?);
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

impl<'a> EntityFactory<'a> {
  pub fn try_union(&self, values: Vec<Entity<'a>>) -> Option<Entity<'a>> {
    if values.is_empty() {
      None
    } else {
      Some(if values.len() == 1 {
        values.first().unwrap().clone()
      } else {
        self.entity(UnionEntity { values, consumed: Cell::new(false) })
      })
    }
  }

  pub fn union(&self, values: Vec<Entity<'a>>) -> Entity<'a> {
    self.try_union(values).unwrap()
  }

  pub fn computed_union<T: ConsumableTrait<'a> + 'a>(
    &self,
    values: Vec<Entity<'a>>,
    dep: T,
  ) -> Entity<'a> {
    self.computed(self.union(values), dep)
  }
}
