use super::{Consumable, InteractionKind, LiteralEntity, TypeofResult, UnionEntity};
use crate::analyzer::Analyzer;
use rustc_hash::FxHashSet;
use std::{fmt::Debug, rc::Rc};

pub trait EntityTrait<'a>: Debug {
  fn consume(&self, analyzer: &mut Analyzer<'a>);
  fn interact(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, kind: InteractionKind);

  fn get_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
  ) -> Entity<'a>;
  fn set_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
    value: Entity<'a>,
  );
  fn enumerate_properties(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)>;
  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: &Entity<'a>);
  fn call(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a>;
  fn r#await(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a>;
  fn iterate(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>);

  fn get_typeof(&self) -> Entity<'a>;
  fn get_to_string(&self, rc: &Entity<'a>) -> Entity<'a>;
  fn get_to_numeric(&self, rc: &Entity<'a>) -> Entity<'a>;
  fn get_to_boolean(&self, rc: &Entity<'a>) -> Entity<'a>;
  fn get_to_property_key(&self, rc: &Entity<'a>) -> Entity<'a>;
  fn get_to_literals(&self) -> Option<FxHashSet<LiteralEntity<'a>>> {
    None
  }
  fn get_literal(&self) -> Option<LiteralEntity<'a>> {
    self.get_to_literals().and_then(
      |set| {
        if set.len() == 1 {
          set.into_iter().next()
        } else {
          None
        }
      },
    )
  }

  fn test_typeof(&self) -> TypeofResult;
  fn test_truthy(&self) -> Option<bool>;
  fn test_nullish(&self) -> Option<bool>;
  fn test_is_undefined(&self) -> Option<bool> {
    let t = self.test_typeof();
    match (t == TypeofResult::Undefined, t.contains(TypeofResult::Undefined)) {
      (true, _) => Some(true),
      (false, true) => None,
      (false, false) => Some(false),
    }
  }
  fn test_is_completely_unknown(&self) -> bool {
    false
  }
}

#[derive(Debug)]
pub struct Entity<'a>(pub Rc<dyn EntityTrait<'a> + 'a>);

impl<'a> Entity<'a> {
  pub fn new(entity: impl EntityTrait<'a> + 'a) -> Self {
    Self(Rc::new(entity))
  }

  pub fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.0.consume(analyzer)
  }

  pub fn interact(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    kind: InteractionKind,
  ) {
    self.0.interact(analyzer, dep.into(), kind)
  }

  pub fn get_property(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    self.0.get_property(self, analyzer, dep.into(), key)
  }

  pub fn set_property(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    self.0.set_property(self, analyzer, dep.into(), key, value)
  }

  pub fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    self.0.enumerate_properties(self, analyzer, dep.into())
  }

  pub fn delete_property(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    key: &Entity<'a>,
  ) {
    self.0.delete_property(analyzer, dep.into(), key)
  }

  pub fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.0.call(self, analyzer, dep.into(), this, args)
  }

  pub fn r#await(&self, analyzer: &mut Analyzer<'a>, dep: impl Into<Consumable<'a>>) -> Entity<'a> {
    self.0.r#await(self, analyzer, dep.into())
  }

  pub fn iterate(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    self.0.iterate(self, analyzer, dep.into())
  }

  pub fn get_typeof(&self) -> Entity<'a> {
    self.0.get_typeof()
  }

  pub fn get_to_string(&self) -> Entity<'a> {
    self.0.get_to_string(self)
  }

  pub fn get_to_numeric(&self) -> Entity<'a> {
    self.0.get_to_numeric(self)
  }

  pub fn get_to_boolean(&self) -> Entity<'a> {
    self.0.get_to_boolean(self)
  }

  pub fn get_to_property_key(&self) -> Entity<'a> {
    self.0.get_to_property_key(self)
  }

  pub fn get_to_literals(&self) -> Option<FxHashSet<LiteralEntity<'a>>> {
    self.0.get_to_literals()
  }

  pub fn get_literal(&self) -> Option<LiteralEntity<'a>> {
    self.0.get_literal()
  }

  pub fn test_typeof(&self) -> TypeofResult {
    self.0.test_typeof()
  }

  pub fn test_truthy(&self) -> Option<bool> {
    self.0.test_truthy()
  }

  pub fn test_nullish(&self) -> Option<bool> {
    self.0.test_nullish()
  }

  pub fn test_is_undefined(&self) -> Option<bool> {
    self.0.test_is_undefined()
  }

  pub fn test_is_completely_unknown(&self) -> bool {
    self.0.test_is_completely_unknown()
  }

  pub fn destruct_as_array(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    length: usize,
  ) -> (Vec<Entity<'a>>, Entity<'a>) {
    let (elements, rest) = self.iterate(analyzer, dep);
    let mut result = Vec::new();
    for i in 0..length.min(elements.len()) {
      result.push(elements[i].clone());
    }
    for _ in 0..length.saturating_sub(elements.len()) {
      if let Some(rest) = rest.clone() {
        result.push(rest.clone());
      } else {
        result.push(LiteralEntity::new_undefined());
      }
    }
    let rest_arr = analyzer.new_empty_array();
    let mut rest_arr_is_empty = true;
    if length < elements.len() {
      for element in &elements[length..elements.len()] {
        rest_arr.push_element(element.clone());
        rest_arr_is_empty = false;
      }
    }
    if let Some(rest) = rest {
      rest_arr.init_rest(rest);
      rest_arr_is_empty = false;
    }
    if rest_arr_is_empty {
      rest_arr.deps.borrow_mut().push(self.clone().into());
    }
    (result, Entity::new(rest_arr))
  }

  pub fn iterate_result_union(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
  ) -> Option<Entity<'a>> {
    let (elements, rest) = self.iterate(analyzer, dep);
    if let Some(rest) = rest {
      let mut result = elements;
      result.push(rest);
      Some(UnionEntity::new(result))
    } else if !elements.is_empty() {
      Some(UnionEntity::new(elements))
    } else {
      None
    }
  }
}

impl<'a> Clone for Entity<'a> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<'a, T: EntityTrait<'a> + 'a> From<T> for Entity<'a> {
  fn from(entity: T) -> Self {
    Self::new(entity)
  }
}
