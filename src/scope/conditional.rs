use super::CfScopeKind;
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable, ConsumableTrait},
  dep::DepId,
  entity::Entity,
};
use std::{cell::Cell, fmt::Debug, mem};

#[derive(Debug, Default)]
pub struct ConditionalData<'a> {
  true_referred: bool,
  false_referred: bool,
  referred_tests: Vec<Entity<'a>>,
}

#[derive(Debug, Clone)]
struct ConditionalBranchConsumable<'a> {
  dep_id: DepId,
  maybe_true: bool,
  maybe_false: bool,
  test: Entity<'a>,
  referred: &'a Cell<bool>,
}

impl<'a> ConsumableTrait<'a> for &'a ConditionalBranchConsumable<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    if !self.referred.get() {
      self.referred.set(true);

      if let Some(data) = analyzer.conditional_data.get_mut(&self.dep_id) {
        data.true_referred |= self.maybe_true;
        data.false_referred |= self.maybe_false;
        data.referred_tests.push(self.test);
      } else {
        // When this conditional scope is already consumed in `post_analyze_handle_conditional`
        // we should consume the test here
        self.test.consume(analyzer);
      }
    }
  }
  fn cloned(&self) -> Consumable<'a> {
    Box::new(*self)
  }
}

impl<'a> Analyzer<'a> {
  pub fn push_conditional_cf_scope(
    &mut self,
    dep_id: impl Into<DepId>,
    kind: CfScopeKind,
    test: Entity<'a>,
    maybe_true: bool,
    maybe_false: bool,
  ) -> impl ConsumableTrait<'a> + 'a {
    let dep_id = dep_id.into();

    self.conditional_data.entry(dep_id).or_insert_with(Default::default);

    let dep: &ConditionalBranchConsumable<'a> = self.allocator.alloc(ConditionalBranchConsumable {
      dep_id,
      maybe_true,
      maybe_false,
      test,
      referred: self.allocator.alloc(Cell::new(false)),
    });

    self.push_cf_scope_with_deps(
      kind,
      None,
      vec![box_consumable(dep)],
      if maybe_true && maybe_false { None } else { Some(false) },
    );

    dep
  }

  pub fn post_analyze_handle_conditional(&mut self) {
    let conditional_data = mem::take(&mut self.conditional_data);

    let mut deps_to_consume = vec![];
    let mut tests_to_consume = vec![];

    for (dep, data) in conditional_data {
      if data.true_referred && data.false_referred {
        deps_to_consume.push(dep);
        tests_to_consume.push(data.referred_tests);
      } else {
        self.conditional_data.insert(dep, data);
      }
    }

    if deps_to_consume.is_empty() {
      return;
    }

    for dep in deps_to_consume {
      self.refer_dep(dep);
    }
    for tests in tests_to_consume {
      for test in tests {
        test.consume(self);
      }
    }

    self.post_analyze_handle_conditional();
  }
}
