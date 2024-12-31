use super::{LiteralEntity, TypeofResult};
use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableTrait},
};
use rustc_hash::FxHashSet;
use std::fmt::Debug;

/// (vec![(definite, key, value)], dep)
pub type EnumeratedProperties<'a> = (Vec<(bool, Entity<'a>, Entity<'a>)>, Consumable<'a>);

/// (vec![known_elements], rest, dep)
pub type IteratedElements<'a> = (Vec<Entity<'a>>, Option<Entity<'a>>, Consumable<'a>);

pub trait EntityTrait<'a>: Debug {
  fn consume(&'a self, analyzer: &mut Analyzer<'a>);
  /// Returns true if the entity is completely consumed
  fn consume_mangable(&'a self, analyzer: &mut Analyzer<'a>) -> bool {
    self.consume(analyzer);
    true
  }
  fn unknown_mutate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>);

  fn get_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a>;
  fn set_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  );
  fn enumerate_properties(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a>;
  fn delete_property(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>);
  fn call(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a>;
  fn construct(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a>;
  fn jsx(&'a self, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a>;
  fn r#await(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> Entity<'a>;
  fn iterate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> IteratedElements<'a>;

  fn get_destructable(&'a self, analyzer: &Analyzer<'a>, dep: Consumable<'a>) -> Consumable<'a>;
  fn get_typeof(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_string(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_numeric(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_boolean(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_property_key(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_jsx_child(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a>;
  fn get_to_literals(&'a self, _analyzer: &Analyzer<'a>) -> Option<FxHashSet<LiteralEntity<'a>>> {
    None
  }
  fn get_literal(&'a self, analyzer: &Analyzer<'a>) -> Option<LiteralEntity<'a>> {
    self.get_to_literals(analyzer).and_then(|set| {
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

  fn destruct_as_array(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    length: usize,
    need_rest: bool,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>, Consumable<'a>) {
    let (elements, rest, deps) = self.iterate(analyzer, dep);
    let mut result_elements = Vec::new();
    for i in 0..length.min(elements.len()) {
      result_elements.push(analyzer.factory.computed(elements[i], deps));
    }
    if elements.len() < length {
      let missing = analyzer.factory.computed(rest.unwrap_or(analyzer.factory.undefined), deps);
      for _ in elements.len()..length {
        result_elements.push(missing);
      }
    }
    let rest_arr = need_rest.then(|| {
      let rest_arr = analyzer.new_empty_array();
      rest_arr.deps.borrow_mut().push(deps);
      let mut rest_arr_is_empty = true;
      if length < elements.len() {
        for element in &elements[length..elements.len()] {
          rest_arr.push_element(*element);
          rest_arr_is_empty = false;
        }
      }
      if let Some(rest) = rest {
        rest_arr.init_rest(rest);
        rest_arr_is_empty = false;
      }
      if rest_arr_is_empty {
        rest_arr.deps.borrow_mut().push(analyzer.consumable(self));
      }
      rest_arr as Entity<'a>
    });
    (result_elements, rest_arr, deps)
  }

  fn iterate_result_union(
    &'a self,
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

  fn call_as_getter(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
  ) -> Entity<'a> {
    self.call(analyzer, dep, this, analyzer.factory.empty_arguments)
  }

  fn call_as_setter(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    value: Entity<'a>,
  ) -> Entity<'a> {
    self.call(analyzer, dep, this, analyzer.factory.arguments(vec![(false, value)]))
  }
}

pub type Entity<'a> = &'a (dyn EntityTrait<'a> + 'a);

impl<'a, T: EntityTrait<'a> + 'a + ?Sized> ConsumableTrait<'a> for &'a T {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    (*self).consume(analyzer)
  }
}
