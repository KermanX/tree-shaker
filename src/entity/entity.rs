use super::{EntityFactory, LiteralEntity, TypeofResult};
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable, ConsumableNode, ConsumableTrait},
};
use oxc::allocator::Allocator;
use rustc_hash::FxHashSet;
use std::fmt::Debug;

/// (vec![(definite, key, value)], dep)
pub type EnumeratedProperties<'a> = (Vec<(bool, Entity<'a>, Entity<'a>)>, Consumable<'a>);

/// (vec![known_elements], rest, dep)
pub type IteratedElements<'a> = (Vec<Entity<'a>>, Option<Entity<'a>>, Consumable<'a>);

pub trait EntityTrait<'a>: Debug {
  fn consume(&self, analyzer: &mut Analyzer<'a>);

  fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a>;
  fn set_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  );
  fn enumerate_properties(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a>;
  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>);
  fn call(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a>;
  fn construct(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a>;
  fn r#await(&self, rc: Entity<'a>, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>)
    -> Entity<'a>;
  fn iterate(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a>;

  fn get_destructable(&self, rc: Entity<'a>, dep: Consumable<'a>) -> Consumable<'a>;
  fn get_typeof(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_string(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_numeric(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_boolean(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_property_key(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_literals(
    &self,
    _rc: Entity<'a>,
    _analyzer: &Analyzer<'a>,
  ) -> Option<FxHashSet<LiteralEntity<'a>>> {
    None
  }
  fn get_literal(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Option<LiteralEntity<'a>> {
    self.get_to_literals(rc, analyzer).and_then(|set| {
      if set.len() == 1 {
        set.into_iter().next()
      } else {
        None
      }
    })
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

#[derive(Debug, Clone, Copy)]
pub struct Entity<'a>(pub &'a (dyn EntityTrait<'a> + 'a));

impl<'a> EntityFactory<'a> {
  pub fn entity(&self, entity: impl EntityTrait<'a> + 'a) -> Entity<'a> {
    Entity::new_in(entity, self.allocator)
  }
}

impl<'a> Entity<'a> {
  pub fn new_in(entity: impl EntityTrait<'a> + 'a, allocator: &'a Allocator) -> Self {
    Self(allocator.alloc(entity))
  }

  pub fn ptr_eq(self, other: Self) -> bool {
    std::ptr::addr_eq(self.0 as *const _, other.0 as *const _)
  }

  pub fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.0.consume(analyzer)
  }

  pub fn get_property(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    self.0.get_property(*self, analyzer, dep.into(), key)
  }

  pub fn set_property(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.0.set_property(*self, analyzer, dep.into(), key, value)
  }

  pub fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
  ) -> EnumeratedProperties<'a> {
    self.0.enumerate_properties(*self, analyzer, dep.into())
  }

  pub fn delete_property(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    key: Entity<'a>,
  ) {
    self.0.delete_property(analyzer, dep.into(), key)
  }

  pub fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.0.call(*self, analyzer, dep.into(), this, args)
  }

  pub fn construct(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.0.construct(*self, analyzer, dep.into(), args)
  }

  pub fn r#await(&self, analyzer: &mut Analyzer<'a>, dep: impl Into<Consumable<'a>>) -> Entity<'a> {
    self.0.r#await(*self, analyzer, dep.into())
  }

  pub fn iterate(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
  ) -> IteratedElements<'a> {
    self.0.iterate(*self, analyzer, dep.into())
  }

  pub fn get_destructable(&self, dep: impl Into<Consumable<'a>>) -> Consumable<'a> {
    self.0.get_destructable(*self, dep.into())
  }

  pub fn get_typeof(&self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.0.get_typeof(*self, analyzer)
  }

  pub fn get_to_string(&self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.0.get_to_string(*self, analyzer)
  }

  pub fn get_to_numeric(&self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.0.get_to_numeric(*self, analyzer)
  }

  pub fn get_to_boolean(&self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.0.get_to_boolean(*self, analyzer)
  }

  pub fn get_to_property_key(&self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.0.get_to_property_key(*self, analyzer)
  }

  pub fn get_to_literals(&self, analyzer: &Analyzer<'a>) -> Option<FxHashSet<LiteralEntity<'a>>> {
    self.0.get_to_literals(*self, analyzer)
  }

  pub fn get_literal(&self, analyzer: &Analyzer<'a>) -> Option<LiteralEntity<'a>> {
    self.0.get_literal(*self, analyzer)
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
    dep: Consumable<'a>,
    length: usize,
  ) -> (Vec<Entity<'a>>, Entity<'a>, Consumable<'a>) {
    let (elements, rest, deps) = self.iterate(analyzer, dep);
    let deps = box_consumable(ConsumableNode::new(deps));
    let mut result_elements = Vec::new();
    for i in 0..length.min(elements.len()) {
      result_elements.push(analyzer.factory.computed(elements[i].clone(), deps.cloned()));
    }
    for _ in 0..length.saturating_sub(elements.len()) {
      if let Some(rest) = rest {
        result_elements.push(analyzer.factory.computed(rest, deps.cloned()));
      } else {
        result_elements.push(analyzer.factory.computed(analyzer.factory.undefined, deps.cloned()));
      }
    }
    let rest_arr = analyzer.new_empty_array();
    rest_arr.deps.borrow_mut().push(deps.cloned());
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
      rest_arr.deps.borrow_mut().push(self.to_consumable());
    }
    (result_elements, analyzer.factory.entity(rest_arr), deps)
  }

  pub fn iterate_result_union(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Option<Entity<'a>> {
    let (elements, rest, deps) = self.iterate(analyzer, dep);
    if let Some(rest) = rest {
      let mut result = elements;
      result.push(rest);
      Some(analyzer.factory.computed_union(result, deps))
    } else if !elements.is_empty() {
      Some(analyzer.factory.computed_union(elements, deps))
    } else {
      None
    }
  }

  pub fn to_consumable(&self) -> Consumable<'a> {
    box_consumable(self.clone())
  }
}

// impl<'a, T: EntityTrait<'a> + 'a> From<T> for Entity<'a> {
//   fn from(entity: T) -> Self {
//     Self::new(entity)
//   }
// }
