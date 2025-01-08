use oxc::ast::ast::LogicalOperator;

use super::{
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, TypeofResult,
};
use crate::{analyzer::Analyzer, consumable::Consumable};

#[derive(Debug, Clone)]
pub struct LogicalResultEntity<'a> {
  pub value: Entity<'a>,
  pub is_coalesce: bool,
  pub result: Option<bool>,
}

impl<'a> EntityTrait<'a> for LogicalResultEntity<'a> {
  fn consume(&'a self, analyzer: &mut Analyzer<'a>) {
    self.value.consume(analyzer);
  }

  fn unknown_mutate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    self.value.unknown_mutate(analyzer, dep);
  }

  fn get_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    self.value.get_property(analyzer, dep, key)
  }

  fn set_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.value.set_property(analyzer, dep, key, value);
  }

  fn enumerate_properties(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    self.value.enumerate_properties(analyzer, dep)
  }

  fn delete_property(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.value.delete_property(analyzer, dep, key);
  }

  fn call(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.value.call(analyzer, dep, this, args)
  }

  fn construct(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.value.construct(analyzer, dep, args)
  }

  fn jsx(&'a self, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    self.value.jsx(analyzer, props)
  }

  fn r#await(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> Entity<'a> {
    self.value.r#await(analyzer, dep)
  }

  fn iterate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> IteratedElements<'a> {
    self.value.iterate(analyzer, dep)
  }

  fn get_destructable(&'a self, analyzer: &Analyzer<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    self.value.get_destructable(analyzer, dep)
  }

  fn get_typeof(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.value.get_typeof(analyzer)
  }

  fn get_to_string(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.value.get_to_string(analyzer)
  }

  fn get_to_numeric(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.value.get_to_numeric(analyzer)
  }

  fn get_to_boolean(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    let value = self.value.get_to_boolean(analyzer);
    if self.is_coalesce {
      value
    } else if let Some(result) = self.result {
      analyzer.factory.computed(analyzer.factory.boolean(result), value)
    } else {
      value
    }
  }

  fn get_to_property_key(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.value.get_to_property_key(analyzer)
  }

  fn get_to_jsx_child(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.value.get_to_jsx_child(analyzer)
  }

  fn test_typeof(&self) -> TypeofResult {
    self.value.test_typeof()
  }

  fn test_truthy(&self) -> Option<bool> {
    if self.is_coalesce {
      self.value.test_truthy()
    } else {
      self.result
    }
  }

  fn test_nullish(&self) -> Option<bool> {
    if self.is_coalesce {
      self.result
    } else {
      self.value.test_nullish()
    }
  }
}

impl<'a> EntityFactory<'a> {
  /// Only used when (maybe_left, maybe_right) == (true, true)
  pub fn logical_result(
    &self,
    left: Entity<'a>,
    right: Entity<'a>,
    operator: LogicalOperator,
  ) -> &'a mut LogicalResultEntity<'a> {
    self.alloc(LogicalResultEntity {
      value: self.union((left, right)),
      is_coalesce: operator == LogicalOperator::Coalesce,
      result: match operator {
        LogicalOperator::Or => match right.test_truthy() {
          Some(true) => Some(true),
          _ => None,
        },
        LogicalOperator::And => match right.test_truthy() {
          Some(false) => Some(false),
          _ => None,
        },
        LogicalOperator::Coalesce => match right.test_nullish() {
          Some(false) => Some(false),
          _ => None,
        },
      },
    })
  }
}
