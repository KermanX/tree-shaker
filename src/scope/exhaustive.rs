use crate::{analyzer::Analyzer, entity::Entity, scope::CfScopeKind};
use oxc::semantic::SymbolId;
use std::{mem, rc::Rc};

impl<'a> Analyzer<'a> {
  pub fn exec_loop(&mut self, runner: impl Fn(&mut Analyzer<'a>) -> () + 'a) {
    self.exec_exhaustively_impl(false, runner)
  }

  pub fn exec_exhaustively(&mut self, runner: impl Fn(&mut Analyzer<'a>) -> () + 'a) {
    self.exec_exhaustively_impl(true, runner)
  }

  fn exec_exhaustively_impl(
    &mut self,
    track_dep_after_finished: bool,
    runner: impl Fn(&mut Analyzer<'a>) -> () + 'a,
  ) {
    self.push_cf_scope(CfScopeKind::Exhaustive, None, Some(false));
    let mut round_counter = 0;
    while self.cf_scope().borrow_mut().iterate_exhaustively() {
      runner(self);
      round_counter += 1;
      if round_counter > 1000 {
        unreachable!("Exhaustive loop is too deep");
      }
    }
    let scope = self.pop_cf_scope();
    if track_dep_after_finished {
      let mut scope_ref = scope.borrow_mut();
      let exhaustive_data = scope_ref.exhaustive_data.as_mut().unwrap();
      let deps = mem::take(&mut exhaustive_data.deps);
      let runner: Rc<dyn Fn(&mut Analyzer<'a>) -> () + 'a> = Rc::new(runner);
      for symbol in deps {
        self.exhaustive_deps.entry(symbol).or_insert_with(Vec::new).push(runner.clone());
      }
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
        let runner = runner.clone();
        (*runner)(self);
      }
    }
  }

  pub fn has_exhaustive_scope_since(&self, target: usize) -> bool {
    self.scope_context.cf_scopes[target..].iter().any(|scope| scope.borrow().is_exhaustive())
  }
}
