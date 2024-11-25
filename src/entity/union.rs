use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  utils::UnionLike,
  value::EntityValueKind,
  Entity, EntityFactory, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable, ConsumableTrait},
  scope::CfScopeKind,
  use_consumed_flag,
};
use rustc_hash::FxHashSet;
use std::{cell::Cell, fmt::Debug};

#[derive(Debug)]
pub struct UnionEntity<'a, V: UnionLike<'a, Entity<'a>> + Debug + 'a> {
  /// Possible values
  pub values: V,
  consumed: Cell<bool>,
  phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, V: UnionLike<'a, Entity<'a>> + Debug + 'a> EntityTrait<'a> for UnionEntity<'a, V> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    for value in self.values.iter() {
      value.consume(analyzer);
    }
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    for value in self.values.iter() {
      value.unknown_mutate(analyzer, dep.cloned());
    }
  }

  fn get_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    let values = analyzer.exec_indeterminately(|analyzer| {
      self.values.map(|v| v.get_property(analyzer, dep.cloned(), key))
    });
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
    analyzer.exec_indeterminately(|analyzer| {
      for entity in self.values.iter() {
        entity.set_property(analyzer, dep.cloned(), key, value.clone())
      }
    });
  }

  fn enumerate_properties(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    // FIXME:
    consumed_object::enumerate_properties(rc, analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    analyzer.exec_indeterminately(|analyzer| {
      for entity in self.values.iter() {
        entity.delete_property(analyzer, dep.cloned(), key);
      }
    })
  }

  fn call(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    analyzer.push_cf_scope_with_deps(CfScopeKind::Dependent, None, vec![box_consumable(rc)], None);
    let values = self.values.map(|v| v.call(analyzer, dep.cloned(), this, args));
    analyzer.pop_cf_scope();
    analyzer.factory.union(values)
  }

  fn construct(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let values = analyzer.exec_indeterminately(|analyzer| {
      self.values.map(|v| v.construct(analyzer, dep.cloned(), args.clone()))
    });
    analyzer.factory.union(values)
  }

  fn jsx(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    let values =
      analyzer.exec_indeterminately(|analyzer| self.values.map(|v| v.jsx(analyzer, props.clone())));
    analyzer.factory.union(values)
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    let values = analyzer
      .exec_indeterminately(|analyzer| self.values.map(|v| v.r#await(analyzer, dep.cloned())));
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
    analyzer.push_indeterminate_cf_scope();
    for entity in self.values.iter() {
      if let Some(result) = entity.iterate_result_union(analyzer, dep.cloned()) {
        results.push(result);
      } else {
        has_undefined = true;
      }
    }
    analyzer.pop_cf_scope();
    if has_undefined {
      results.push(analyzer.factory.undefined);
    }
    (vec![], analyzer.factory.try_union(results), box_consumable(()))
  }

  fn get_value(&self) -> EntityValueKind<'a> {
    let mut result = vec![];
    for entity in self.values.iter() {
      match entity.get_value() {
        EntityValueKind::Unknown => return EntityValueKind::Unknown,
        value => result.push(value),
      }
    }
    EntityValueKind::Union(result)
  }

  fn get_destructable(&self, _rc: Entity<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    let mut values = Vec::new();
    for entity in self.values.iter() {
      values.push(entity.get_destructable(dep.cloned()));
    }
    box_consumable(values)
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    // TODO: collect literals
    let values = self.values.map(|v| v.get_typeof(analyzer));
    analyzer.factory.union(values)
  }

  fn get_to_string(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    // TODO: dedupe
    let values = self.values.map(|v| v.get_to_string(analyzer));
    analyzer.factory.union(values)
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    // TODO: dedupe
    let values = self.values.map(|v| v.get_to_numeric(analyzer));
    analyzer.factory.union(values)
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    let values = self.values.map(|v| v.get_to_boolean(analyzer));
    analyzer.factory.union(values)
  }

  fn get_to_property_key(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    let values = self.values.map(|v| v.get_to_property_key(analyzer));
    analyzer.factory.union(values)
  }

  fn get_to_jsx_child(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    let values = self.values.map(|v| v.get_to_jsx_child(analyzer));
    analyzer.factory.union(values)
  }

  fn get_to_literals(
    &self,
    _rc: Entity<'a>,
    analyzer: &Analyzer<'a>,
  ) -> Option<FxHashSet<LiteralEntity<'a>>> {
    let mut iter = self.values.iter();
    let mut result = iter.next().unwrap().get_to_literals(analyzer)?;
    for entity in iter {
      result.extend(entity.get_to_literals(analyzer)?);
    }
    Some(result)
  }

  fn test_typeof(&self) -> TypeofResult {
    let mut result = TypeofResult::_None;
    for entity in self.values.iter() {
      result |= entity.test_typeof();
    }
    result
  }

  fn test_truthy(&self) -> Option<bool> {
    let mut iter = self.values.iter();
    let result = iter.next().unwrap().test_truthy()?;
    for entity in iter {
      if entity.test_truthy()? != result {
        return None;
      }
    }
    Some(result)
  }

  fn test_nullish(&self) -> Option<bool> {
    let mut iter = self.values.iter();
    let result = iter.next().unwrap().test_nullish()?;
    for entity in iter {
      if entity.test_nullish()? != result {
        return None;
      }
    }
    Some(result)
  }
}

impl<'a> EntityFactory<'a> {
  pub fn try_union<V: UnionLike<'a, Entity<'a>> + Debug + 'a>(
    &self,
    values: V,
  ) -> Option<Entity<'a>> {
    if values.len() == 0 {
      None
    } else {
      Some(if values.len() == 1 {
        values.iter().next().unwrap()
      } else {
        self.entity(UnionEntity {
          values,
          consumed: Cell::new(false),
          phantom: std::marker::PhantomData,
        })
      })
    }
  }

  pub fn union<V: UnionLike<'a, Entity<'a>> + Debug + 'a>(&self, values: V) -> Entity<'a> {
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
