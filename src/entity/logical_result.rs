use oxc::ast::ast::LogicalOperator;

use super::{
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityTrait, TypeofResult,
};
use crate::{analyzer::Analyzer, consumable::Consumable};

#[derive(Debug, Clone)]
pub struct LogicalResultEntity<'a> {
  pub value: Entity<'a>,
  pub is_coalesce: bool,
  pub result: Option<bool>,
}

impl<'a> EntityTrait<'a> for LogicalResultEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.value.consume(analyzer);
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    self.value.unknown_mutate(analyzer, dep);
  }

  fn get_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    self.value.get_property(analyzer, dep, key)
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.value.set_property(analyzer, dep, key, value);
  }

  fn enumerate_properties(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    self.value.enumerate_properties(analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.value.delete_property(analyzer, dep, key);
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.value.call(analyzer, dep, this, args)
  }

  fn construct(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.value.construct(analyzer, dep, args)
  }

  fn jsx(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    self.value.jsx(analyzer, props)
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    self.value.r#await(analyzer, dep)
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    self.value.iterate(analyzer, dep)
  }

  fn get_destructable(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Consumable<'a> {
    self.value.get_destructable(analyzer, dep)
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value.get_typeof(analyzer)
  }

  fn get_to_string(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value.get_to_string(analyzer)
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value.get_to_numeric(analyzer)
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    let value = self.value.get_to_boolean(analyzer);
    if self.is_coalesce {
      value
    } else if let Some(result) = self.result {
      analyzer.factory.computed(analyzer.factory.boolean(result), value)
    } else {
      value
    }
  }

  fn get_to_property_key(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value.get_to_property_key(analyzer)
  }

  fn get_to_jsx_child(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value.get_to_jsx_child(analyzer)
  }

  fn test_typeof(&self, analyzer: &mut Analyzer<'a>) -> TypeofResult {
    self.value.test_typeof(analyzer)
  }

  fn test_truthy(&self, analyzer: &mut Analyzer<'a>) -> Option<bool> {
    if self.is_coalesce {
      self.value.test_truthy(analyzer)
    } else {
      self.result
    }
  }

  fn test_nullish(&self, analyzer: &mut Analyzer<'a>) -> Option<bool> {
    if self.is_coalesce {
      self.result
    } else {
      self.value.test_nullish(analyzer)
    }
  }
}

impl<'a> Analyzer<'a> {
  /// Only used when (maybe_left, maybe_right) == (true, true)
  pub fn logical_result(
    &mut self,
    left: Entity<'a>,
    right: Entity<'a>,
    operator: LogicalOperator,
  ) -> Entity<'a> {
    self.factory.entity(LogicalResultEntity {
      value: self.factory.union((left, right)),
      is_coalesce: operator == LogicalOperator::Coalesce,
      result: match operator {
        LogicalOperator::Or => match right.test_truthy(self) {
          Some(true) => Some(true),
          _ => None,
        },
        LogicalOperator::And => match right.test_truthy(self) {
          Some(false) => Some(false),
          _ => None,
        },
        LogicalOperator::Coalesce => match right.test_nullish(self) {
          Some(false) => Some(false),
          _ => None,
        },
      },
    })
  }
}
