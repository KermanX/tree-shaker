use super::{
  collected::CollectedEntity,
  consumed_object,
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  interactions::InteractionKind,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::UnknownEntity,
};
use crate::{analyzer::Analyzer, scope::CfScopeKind};
use rustc_hash::FxHashSet;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct UnionEntity<'a> {
  /// Possible values
  pub values: Vec<Entity<'a>>,
  /// Which is the actual value depends on these
  pub deps: Rc<Vec<Entity<'a>>>,
}

impl<'a> EntityTrait<'a> for UnionEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    for value in &self.values {
      value.consume(analyzer);
    }
    for dep in self.deps.iter() {
      dep.consume(analyzer);
    }
  }

  fn interact(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep, kind: InteractionKind) {
    for value in &self.values {
      value.interact(analyzer, dep.clone(), kind);
    }
  }

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    let mut values = Vec::new();
    for entity in &self.values {
      values.push(entity.get_property(analyzer, dep.clone(), key));
    }
    UnionEntity::new_with_deps(values, self.deps.clone())
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    for entity in &self.values {
      entity.set_property(analyzer, dep.clone(), key, value.clone());
    }
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    // FIXME:
    self.consume(analyzer);
    consumed_object::enumerate_properties(analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep, key: &Entity<'a>) {
    for entity in &self.values {
      entity.delete_property(analyzer, dep.clone(), key);
    }
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    let mut results = Vec::new();
    analyzer.push_cf_scope(CfScopeKind::Normal, None, None);
    for entity in &self.values {
      results.push(entity.call(analyzer, dep.clone(), this, args));
    }
    analyzer.pop_cf_scope();
    UnionEntity::new_with_deps(results, self.deps.clone())
  }

  fn r#await(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    let mut values = Vec::new();
    for entity in &self.values {
      values.push(entity.r#await(analyzer));
    }
    UnionEntity::new_with_deps(values, self.deps.clone())
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    let mut results = Vec::new();
    let mut has_undefined = false;
    for entity in &self.values {
      if let Some(result) = entity.iterate_result_union(analyzer, dep.clone()) {
        results.push(result);
      } else {
        has_undefined = true;
      }
    }
    if has_undefined {
      results.push(LiteralEntity::new_undefined());
    }
    (vec![], UnionEntity::try_new_with_deps(results, self.deps.clone()))
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
  pub fn try_new_with_deps(
    values: Vec<Entity<'a>>,
    deps: Rc<Vec<Entity<'a>>>,
  ) -> Option<Entity<'a>> {
    if values.is_empty() {
      None
    } else {
      Some(if values.len() == 1 {
        let value = values.first().unwrap().clone();
        if deps.is_empty() {
          value
        } else {
          CollectedEntity::new(value, RefCell::new((*deps).clone()))
        }
      } else {
        let has_unknown = values.iter().any(|entity| entity.test_is_completely_unknown());
        if has_unknown {
          UnknownEntity::new_unknown_with_deps(values)
        } else {
          Entity::new(UnionEntity { values, deps })
        }
      })
    }
  }

  pub fn new_with_deps(
    values: Vec<Entity<'a>>,
    deps: impl Into<Rc<Vec<Entity<'a>>>>,
  ) -> Entity<'a> {
    Self::try_new_with_deps(values, deps.into()).unwrap()
  }

  pub fn new(values: Vec<Entity<'a>>) -> Entity<'a> {
    Self::new_with_deps(values, Rc::new(vec![]))
  }
}
