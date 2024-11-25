use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  value::EntityValueKind,
  Entity, EntityFactory, EntityTrait, TypeofResult,
};
use crate::{analyzer::Analyzer, consumable::Consumable, use_consumed_flag};
use std::cell::Cell;

#[derive(Debug, Default)]
pub struct ArgumentsEntity<'a> {
  consumed: Cell<bool>,
  pub arguments: Vec<(bool, Entity<'a>)>,
}

impl<'a> EntityTrait<'a> for ArgumentsEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    for (_, entity) in &self.arguments {
      entity.consume(analyzer);
    }
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    if self.consumed.get() {
      return consumed_object::unknown_mutate(analyzer, dep);
    }

    for (_, entity) in &self.arguments {
      entity.unknown_mutate(analyzer, dep.cloned());
    }
  }

  fn get_property(
    &self,
    _rc: Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
    _key: Entity<'a>,
  ) -> Entity<'a> {
    unreachable!()
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
    _key: Entity<'a>,
    _value: Entity<'a>,
  ) {
    unreachable!()
  }

  fn enumerate_properties(
    &self,
    _rc: Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    unreachable!()
  }

  fn delete_property(&self, _analyzer: &mut Analyzer<'a>, _dep: Consumable<'a>, _key: Entity<'a>) {
    unreachable!()
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
    _this: Entity<'a>,
    _args: Entity<'a>,
  ) -> Entity<'a> {
    unreachable!()
  }

  fn construct(
    &self,
    _rc: Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
    _args: Entity<'a>,
  ) -> Entity<'a> {
    unreachable!()
  }

  fn jsx(&self, _rc: Entity<'a>, _analyzer: &mut Analyzer<'a>, _props: Entity<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
  ) -> Entity<'a> {
    unreachable!()
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    let mut elements = Vec::new();
    let mut rest: Option<Vec<Entity<'a>>> = None;
    for (spread, entity) in &self.arguments {
      if *spread {
        if let Some(iterated) = entity.iterate_result_union(analyzer, dep.cloned()) {
          if let Some(rest) = &mut rest {
            rest.push(iterated);
          } else {
            rest = Some(vec![iterated]);
          }
        }
      } else {
        if let Some(rest) = &mut rest {
          rest.push(entity.clone());
        } else {
          elements.push(entity.clone());
        }
      }
    }
    (elements, rest.map(|val| analyzer.factory.union(val)), dep)
  }

  fn get_value(&self) -> EntityValueKind<'a> {
    unreachable!()
  }

  fn get_destructable(&self, _rc: Entity<'a>, _dep: Consumable<'a>) -> Consumable<'a> {
    unreachable!()
  }

  fn get_typeof(&self, _rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_string(&self, _rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_property_key(&self, _rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_jsx_child(&self, _rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn test_typeof(&self) -> TypeofResult {
    unreachable!()
  }

  fn test_truthy(&self) -> Option<bool> {
    unreachable!()
  }

  fn test_nullish(&self) -> Option<bool> {
    unreachable!()
  }
}

impl<'a> EntityFactory<'a> {
  pub fn arguments(&self, arguments: Vec<(bool, Entity<'a>)>) -> Entity<'a> {
    self.entity(ArgumentsEntity { consumed: Cell::new(false), arguments })
  }
}
