use super::{Entity, EntityTrait, TypeofResult, UnionEntity};
use crate::{analyzer::Analyzer, consumable::Consumable};

#[derive(Debug)]
pub struct ArgumentsEntity<'a> {
  pub arguments: Vec<(bool, Entity<'a>)>,
}

impl<'a> EntityTrait<'a> for ArgumentsEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    for (_, entity) in &self.arguments {
      entity.consume(analyzer);
    }
  }

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
    _key: &Entity<'a>,
  ) -> Entity<'a> {
    unreachable!()
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
    _key: &Entity<'a>,
    _value: Entity<'a>,
  ) {
    unreachable!()
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    unreachable!()
  }

  fn delete_property(&self, _analyzer: &mut Analyzer<'a>, _dep: Consumable<'a>, _key: &Entity<'a>) {
    unreachable!()
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
    _this: &Entity<'a>,
    _args: &Entity<'a>,
  ) -> Entity<'a> {
    unreachable!()
  }

  fn r#await(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
  ) -> Entity<'a> {
    unreachable!()
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
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
    (elements, rest.map(UnionEntity::new))
  }

  fn get_typeof(&self) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_numeric(&self, _rc: &Entity<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_boolean(&self, _rc: &Entity<'a>) -> Entity<'a> {
    unreachable!()
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
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

impl<'a> ArgumentsEntity<'a> {
  pub fn new(arguments: Vec<(bool, Entity<'a>)>) -> Entity<'a> {
    Entity::new(Self { arguments })
  }
}
