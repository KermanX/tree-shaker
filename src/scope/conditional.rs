use super::CfScopeKind;
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable, ConsumableTrait},
  dep::DepId,
  entity::Entity,
};
use rustc_hash::FxHashMap;
use std::{cell::Cell, fmt::Debug, mem};

#[derive(Debug, Default)]
struct ConditionalData<'a> {
  maybe_true: bool,
  maybe_false: bool,
  impure_true: bool,
  impure_false: bool,
  referred_tests: Vec<Entity<'a>>,
}

#[derive(Debug, Default)]
pub struct ConditionalDataMap<'a> {
  call_to_deps: FxHashMap<DepId, Vec<&'a ConditionalBranchConsumable<'a>>>,
  node_to_data: FxHashMap<DepId, ConditionalData<'a>>,
}

#[derive(Debug, Clone)]
struct ConditionalBranchConsumable<'a> {
  dep_id: DepId,
  is_true: bool,
  maybe_true: bool,
  maybe_false: bool,
  test: Entity<'a>,
  referred: &'a Cell<bool>,
}

impl<'a> ConsumableTrait<'a> for &'a ConditionalBranchConsumable<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    if !self.referred.get() {
      self.referred.set(true);

      if let Some(data) = analyzer.conditional_data.node_to_data.get_mut(&self.dep_id) {
        data.maybe_true |= self.maybe_true;
        data.maybe_false |= self.maybe_false;
        data.referred_tests.push(self.test);
        if self.is_true {
          data.impure_true = true;
        } else {
          data.impure_false = true;
        }
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
  pub fn push_if_like_branch_cf_scope(
    &mut self,
    dep_id: impl Into<DepId>,
    kind: CfScopeKind,
    test: Entity<'a>,
    maybe_consequent: bool,
    maybe_alternate: bool,
    is_consequent: bool,
    has_alternate: bool,
  ) -> impl ConsumableTrait<'a> + 'a {
    self.push_conditional_cf_scope(
      dep_id,
      kind,
      test,
      maybe_consequent,
      maybe_alternate,
      is_consequent,
      has_alternate,
    )
  }

  pub fn push_logical_right_cf_cope(
    &mut self,
    dep_id: impl Into<DepId>,
    test: Entity<'a>,
    may_enter: bool,
    may_short_circuit: bool,
  ) -> impl ConsumableTrait<'a> + 'a {
    self.push_conditional_cf_scope(
      dep_id,
      CfScopeKind::LogicalRight,
      test,
      may_enter,
      may_short_circuit,
      true,
      false,
    )
  }

  fn push_conditional_cf_scope(
    &mut self,
    dep_id: impl Into<DepId>,
    kind: CfScopeKind,
    test: Entity<'a>,
    maybe_true: bool,
    maybe_false: bool,
    is_true: bool,
    has_contra: bool,
  ) -> impl ConsumableTrait<'a> + 'a {
    let dep_id = dep_id.into();
    let call_id = self.call_scope().dep_id;

    let ConditionalDataMap { call_to_deps, node_to_data } = &mut self.conditional_data;

    let dep: &'a ConditionalBranchConsumable<'a> =
      self.allocator.alloc(ConditionalBranchConsumable {
        dep_id,
        is_true,
        maybe_true,
        maybe_false,
        test,
        referred: self.allocator.alloc(Cell::new(false)),
      });

    if has_contra {
      call_to_deps.entry(call_id).or_insert_with(Default::default).push(dep);
    }
    node_to_data.entry(dep_id).or_insert_with(Default::default);

    self.push_cf_scope_with_deps(
      kind,
      None,
      vec![box_consumable(dep)],
      if maybe_true && maybe_false { None } else { Some(false) },
    );

    dep
  }

  fn is_contra_branch_impure(&self, branch: &'a ConditionalBranchConsumable<'a>) -> bool {
    if let Some(data) = &self.conditional_data.node_to_data.get(&branch.dep_id) {
      if branch.is_true {
        data.impure_false
      } else {
        data.impure_true
      }
    } else {
      false
    }
  }

  pub fn post_analyze_handle_conditional(&mut self) {
    for (call_id, deps) in mem::take(&mut self.conditional_data.call_to_deps) {
      if self.is_referred(call_id) {
        let mut deps_to_consume = vec![];
        for branch in deps {
          if self.is_contra_branch_impure(branch) {
            branch.consume(self);
          } else {
            deps_to_consume.push(branch);
          }
        }
        if !deps_to_consume.is_empty() {
          self.conditional_data.call_to_deps.insert(call_id, deps_to_consume);
        }
      } else {
        self.conditional_data.call_to_deps.insert(call_id, deps);
      }
    }

    let mut deps_to_consume = vec![];
    let mut tests_to_consume = vec![];
    for (dep, data) in mem::take(&mut self.conditional_data.node_to_data) {
      if data.maybe_true && data.maybe_false {
        deps_to_consume.push(dep);
        tests_to_consume.push(data.referred_tests);
      } else {
        self.conditional_data.node_to_data.insert(dep, data);
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
