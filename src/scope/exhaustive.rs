use crate::{analyzer::Analyzer, entity::Entity, scope::CfScopeKind};
use oxc::semantic::SymbolId;
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
    self.exec_exhaustively(Rc::new(runner), false);
  }

  pub fn exec_consumed_fn(&mut self, runner: impl Fn(&mut Analyzer<'a>) -> Entity<'a> + 'a) {
    let runner: Rc<dyn Fn(&mut Analyzer<'a>) -> () + 'a> = Rc::new(move |analyzer| {
      analyzer.push_cf_scope_normal(None);
      analyzer.push_try_scope();
      let ret_val = runner(analyzer);
      ret_val.consume(analyzer);
      analyzer.pop_try_scope().thrown_val().map(|thrown_val| {
        thrown_val.consume(analyzer);
      });
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
  ) -> FxHashSet<SymbolId> {
    self.push_cf_scope(CfScopeKind::Exhaustive, None, Some(false));
    let mut round_counter = 0;
    while self.cf_scope().borrow_mut().iterate_exhaustively() {
      runner(self);
      round_counter += 1;
      if once {
        break;
      }
      if round_counter > 1000 {
        unreachable!("Exhaustive loop is too deep");
      }
    }
    let scope = self.pop_cf_scope();
    let mut scope_ref = scope.borrow_mut();
    let exhaustive_data = scope_ref.exhaustive_data.as_mut().unwrap();
    mem::take(&mut exhaustive_data.deps)
  }

  fn track_dep_after_finished(
    &mut self,
    once: bool,
    runner: Rc<dyn Fn(&mut Analyzer<'a>) -> () + 'a>,
    deps: FxHashSet<SymbolId>,
  ) {
    for symbol in deps {
      self
        .exhaustive_deps
        .entry(symbol)
        .or_insert_with(Default::default)
        .insert(TrackerRunner { runner: runner.clone(), once });
    }
  }

  pub fn mark_exhaustive_read(&mut self, val: &Entity<'a>, symbol: SymbolId, target: usize) {
    if !val.test_is_completely_unknown() {
      for scope in &mut self.scope_context.cf_scopes[target..] {
        scope.borrow_mut().mark_exhaustive_read(symbol)
      }
    }
  }

  pub fn mark_exhaustive_write(&mut self, symbol: SymbolId, target: usize) -> bool {
    let mut should_consume = false;
    for scope in &mut self.scope_context.cf_scopes[target..] {
      should_consume |= scope.borrow_mut().mark_exhaustive_write(symbol)
    }
    should_consume
  }

  pub fn exec_exhaustive_deps(&mut self, should_consume: bool, symbol: SymbolId) {
    if let Some(runners) = self.exhaustive_deps.get_mut(&symbol) {
      let runners = if should_consume { mem::take(runners) } else { runners.clone() };
      for runner in runners {
        let TrackerRunner { runner, once } = runner.clone();
        self.exec_exhaustively(runner, once);
      }
    }
  }

  pub fn has_exhaustive_scope_since(&self, target: usize) -> bool {
    self.scope_context.cf_scopes[target..].iter().any(|scope| scope.borrow().is_exhaustive())
  }
}
