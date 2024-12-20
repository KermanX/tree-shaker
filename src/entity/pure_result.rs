use super::{
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityTrait, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableNode},
  dep::ReferredDeps,
};
use oxc::ast::ast::{CallExpression, NewExpression};
use std::cell::{Cell, RefCell};

#[derive(Debug)]
pub enum PureCallNode<'a> {
  CallExpression(&'a CallExpression<'a>, (Entity<'a>, Entity<'a>, Entity<'a>)),
  NewExpression(&'a NewExpression<'a>, Entity<'a>),
}

#[derive(Debug)]
pub struct PureResult<'a> {
  node: PureCallNode<'a>,
  result: Cell<Option<Entity<'a>>>,
  referred_deps: RefCell<Option<&'a mut ReferredDeps>>,
}

impl<'a> EntityTrait<'a> for PureResult<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.value(analyzer).consume(analyzer);
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    self.value(analyzer).unknown_mutate(analyzer, dep);
  }

  fn get_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    self.value(analyzer).get_property(analyzer, dep, key)
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.value(analyzer).set_property(analyzer, dep, key, value);
  }

  fn enumerate_properties(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    self.value(analyzer).enumerate_properties(analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.value(analyzer).delete_property(analyzer, dep, key);
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.value(analyzer).call(analyzer, dep, this, args)
  }

  fn construct(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.value(analyzer).construct(analyzer, dep, args)
  }

  fn jsx(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    self.value(analyzer).jsx(analyzer, props)
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    self.value(analyzer).r#await(analyzer, dep)
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    self.value(analyzer).iterate(analyzer, dep)
  }

  fn get_destructable(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Consumable<'a> {
    self.value(analyzer).get_destructable(analyzer, dep)
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value(analyzer).get_typeof(analyzer)
  }

  fn get_to_string(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value(analyzer).get_to_string(analyzer)
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value(analyzer).get_to_numeric(analyzer)
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value(analyzer).get_to_boolean(analyzer)
  }

  fn get_to_property_key(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value(analyzer).get_to_property_key(analyzer)
  }

  fn get_to_jsx_child(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.value(analyzer).get_to_jsx_child(analyzer)
  }

  fn test_typeof(&self, analyzer: &mut Analyzer<'a>) -> TypeofResult {
    self.value(analyzer).test_typeof(analyzer)
  }

  fn test_truthy(&self, analyzer: &mut Analyzer<'a>) -> Option<bool> {
    self.value(analyzer).test_truthy(analyzer)
  }

  fn test_nullish(&self, analyzer: &mut Analyzer<'a>) -> Option<bool> {
    self.value(analyzer).test_nullish(analyzer)
  }
}

impl<'a> PureResult<'a> {
  fn exec(&self, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    match &self.node {
      PureCallNode::CallExpression(node, cache) => {
        let result = analyzer.exec_call_expression_in_chain(node, Some(*cache));
        match result {
          Ok((scope_count, value, undefined)) => {
            analyzer.pop_multiple_cf_scopes(scope_count);
            analyzer.factory.optional_union(value, undefined)
          }
          Err(value) => value,
        }
      }
      PureCallNode::NewExpression(node, arguments) => {
        analyzer.exec_new_expression(node, Some(*arguments))
      }
    }
  }

  fn value(&self, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    if let Some(result) = self.result.get() {
      result
    } else {
      let result = if let Some(referred_deps) = self.referred_deps.take() {
        let (val, this_referred_deps) =
          analyzer.exec_in_pure(|analyzer| self.exec(analyzer), referred_deps);
        analyzer.factory.computed(val, ConsumableNode::new(this_referred_deps))
      } else {
        self.exec(analyzer)
      };
      self.result.set(Some(result));
      result
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn pure_result(
    &mut self,
    node: PureCallNode<'a>,
    referred_deps: &'a mut ReferredDeps,
  ) -> Entity<'a> {
    let x = PureResult {
      node,
      result: Cell::new(None),
      referred_deps: RefCell::new(Some(referred_deps)),
    };
    x.value(self);
    self.factory.entity(x)
  }
}
