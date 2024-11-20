use super::cf_scope::ReferredState;
use crate::{analyzer::Analyzer, entity::Entity, scope::CfScopeKind};
use oxc::semantic::{ScopeId, SymbolId};
use rustc_hash::FxHashSet;
use std::{
  hash::{Hash, Hasher},
  mem,
  rc::Rc,
};

#[derive(Clone)]
pub struct TrackerRunner<'a> {
  pub runner: Rc<dyn Fn(&mut Analyzer<'a>) -> () + 'a>,
  pub once: bool,
}
impl<'a> PartialEq for TrackerRunner<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.once == other.once && Rc::ptr_eq(&self.runner, &other.runner)
  }
}
impl<'a> Eq for TrackerRunner<'a> {}
impl Hash for TrackerRunner<'_> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    Rc::as_ptr(&self.runner).hash(state);
  }
}

impl<'a> Analyzer<'a> {
  pub fn exec_loop(&mut self, runner: impl Fn(&mut Analyzer<'a>) -> () + 'a) {
    let runner = Rc::new(runner);

    self.exec_exhaustively(runner.clone(), false);

    let cf_scope = self.cf_scope();
    if cf_scope.referred_state != ReferredState::ReferredClean && cf_scope.deps.may_not_referred() {
      self.push_indeterminate_cf_scope();
      runner(self);
      self.pop_cf_scope();
    }
  }

  pub fn exec_consumed_fn(&mut self, runner: impl Fn(&mut Analyzer<'a>) -> Entity<'a> + 'a) {
    let runner: Rc<dyn Fn(&mut Analyzer<'a>) -> () + 'a> = Rc::new(move |analyzer| {
      analyzer.push_indeterminate_cf_scope();
      analyzer.push_try_scope();
      let ret_val = runner(analyzer);
      let thrown_val = analyzer.pop_try_scope().thrown_val(analyzer);
      if !analyzer.is_inside_pure() {
        analyzer.consume(ret_val);
        analyzer.consume(thrown_val);
      }
      analyzer.pop_cf_scope();
    });
    let deps = self.exec_exhaustively(runner.clone(), false);
    self.track_dep_after_finished(false, runner, deps);
  }

  pub fn exec_async_or_generator_fn(&mut self, runner: impl Fn(&mut Analyzer<'a>) -> () + 'a) {
    let runner = Rc::new(runner);
    let deps = self.exec_exhaustively(runner.clone(), true);
    self.track_dep_after_finished(true, runner, deps);
  }

  fn exec_exhaustively(
    &mut self,
    runner: Rc<dyn Fn(&mut Analyzer<'a>) -> () + 'a>,
    once: bool,
  ) -> FxHashSet<(ScopeId, SymbolId)> {
    self.push_cf_scope(CfScopeKind::Exhaustive, None, Some(false));
    let mut round_counter = 0;
    while self.cf_scope_mut().iterate_exhaustively() {
      runner(self);
      round_counter += 1;
      if once {
        break;
      }
      if round_counter > 1000 {
        unreachable!("Exhaustive loop is too deep");
      }
    }
    let id = self.pop_cf_scope();
    let exhaustive_data = self.scope_context.cf.get_mut(id).exhaustive_data.as_mut().unwrap();
    mem::take(&mut exhaustive_data.deps)
  }

  fn track_dep_after_finished(
    &mut self,
    once: bool,
    runner: Rc<dyn Fn(&mut Analyzer<'a>) -> () + 'a>,
    deps: FxHashSet<(ScopeId, SymbolId)>,
  ) {
    for (scope, symbol) in deps {
      self
        .scope_context
        .variable
        .get_mut(scope)
        .exhaustive_deps
        .entry(symbol)
        .or_insert_with(Default::default)
        .insert(TrackerRunner { runner: runner.clone(), once });
    }
  }

  pub fn mark_exhaustive_read(&mut self, variable: (ScopeId, SymbolId), target: usize) {
    for depth in target..self.scope_context.cf.stack.len() {
      self.scope_context.cf.get_mut_from_depth(depth).mark_exhaustive_read(variable);
    }
  }

  pub fn mark_exhaustive_write(
    &mut self,
    variable: (ScopeId, SymbolId),
    target: usize,
  ) -> (bool, bool) {
    let mut should_consume = false;
    let mut indeterminate = false;
    for depth in target..self.scope_context.cf.stack.len() {
      let scope = self.scope_context.cf.get_mut_from_depth(depth);
      if !should_consume {
        should_consume |= scope.mark_exhaustive_write(variable);
      }
      indeterminate |= scope.is_indeterminate();
    }
    (should_consume, indeterminate)
  }

  pub fn exec_exhaustive_deps(
    &mut self,
    should_consume: bool,
    (scope, symbol): (ScopeId, SymbolId),
  ) -> bool {
    if let Some(runners) =
      self.scope_context.variable.get_mut(scope).exhaustive_deps.get_mut(&symbol)
    {
      let runners = if should_consume { mem::take(runners) } else { runners.clone() };
      if runners.is_empty() {
        false
      } else {
        for runner in runners {
          let TrackerRunner { runner, once } = runner.clone();
          let deps = self.exec_exhaustively(runner.clone(), once);
          self.track_dep_after_finished(once, runner, deps);
        }
        true
      }
    } else {
      false
    }
  }

  pub fn has_exhaustive_scope_since(&self, target_depth: usize) -> bool {
    self.scope_context.cf.iter_stack_range(target_depth..).any(|scope| scope.is_exhaustive())
  }
}
