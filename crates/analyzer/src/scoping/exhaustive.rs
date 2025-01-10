use super::cf_scope::ReferredState;
use crate::{scoping::CfScopeKind, EcmaAnalyzer};
use oxc::semantic::{ScopeId, SymbolId};
use rustc_hash::FxHashSet;
use std::{
  hash::{Hash, Hasher},
  mem,
  rc::Rc,
};

#[derive(Clone)]
pub struct ExhaustiveCallback<'a, A: EcmaAnalyzer<'a> + ?Sized> {
  pub handler: Rc<dyn Fn(&mut A) + 'a>,
  pub once: bool,
}
impl<'a, A: EcmaAnalyzer<'a> + ?Sized> PartialEq for ExhaustiveCallback<'a, A> {
  fn eq(&self, other: &Self) -> bool {
    self.once == other.once && Rc::ptr_eq(&self.handler, &other.handler)
  }
}
impl<'a, A: EcmaAnalyzer<'a> + ?Sized> Eq for ExhaustiveCallback<'a, A> {}
impl<'a, A: EcmaAnalyzer<'a> + ?Sized> Hash for ExhaustiveCallback<'a, A> {
  fn hash<S: Hasher>(&self, state: &mut S) {
    Rc::as_ptr(&self.handler).hash(state);
  }
}

pub trait ExhaustiveScopeAnalyzer<'a> {
  fn exec_loop(&mut self, runner: impl Fn(&mut Self) + 'a)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let runner = Rc::new(runner);

    self.exec_exhaustively("loop", runner.clone(), false);

    let cf_scope = self.cf_scope();
    if cf_scope.referred_state != ReferredState::ReferredClean && cf_scope.deps.may_not_referred() {
      self.push_indeterminate_cf_scope();
      runner(self);
      self.pop_cf_scope();
    }
  }

  fn exec_consumed_fn(&mut self, kind: &str, runner: impl Fn(&mut Self) -> Self::Entity + 'a)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let runner: Rc<dyn Fn(&mut Self) + 'a> = Rc::new(move |analyzer| {
      analyzer.push_indeterminate_cf_scope();
      analyzer.push_try_scope();
      let ret_val = runner(analyzer);
      let thrown_val = analyzer.get_thrown_val(analyzer.pop_try_scope());
      if !analyzer.is_inside_pure() {
        analyzer.consume(ret_val);
        analyzer.consume(thrown_val);
      }
      analyzer.pop_cf_scope();
    });
    let deps = self.exec_exhaustively(kind, runner.clone(), false);
    self.register_exhaustive_callbacks(false, runner, deps);
  }

  fn exec_async_or_generator_fn(&mut self, runner: impl Fn(&mut Self) + 'a)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let runner = Rc::new(runner);
    let deps = self.exec_exhaustively("async/generator", runner.clone(), true);
    self.register_exhaustive_callbacks(true, runner, deps);
  }

  fn exec_exhaustively(
    &mut self,
    kind: &str,
    runner: Rc<dyn Fn(&mut Self) + 'a>,
    once: bool,
  ) -> FxHashSet<(ScopeId, SymbolId)>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.push_cf_scope(CfScopeKind::Exhaustive, None, Some(false));
    let mut round_counter = 0;
    while self.cf_scope_mut().iterate_exhaustively() {
      #[cfg(feature = "flame")]
      let _scope_guard = flame::start_guard(format!(
        "!{kind}@{:06X} x{}",
        (Rc::as_ptr(&runner) as *const () as usize) & 0xFFFFFF,
        round_counter
      ));

      runner(self);
      round_counter += 1;
      if once {
        self.cf_scope_mut().exhaustive_data.as_mut().unwrap().dirty = false;
        break;
      }
      if round_counter > 1000 {
        unreachable!("Exhaustive loop is too deep");
      }
    }
    let id = self.pop_cf_scope();
    let exhaustive_data = self.scoping_mut().cf.get_mut(id).exhaustive_data.as_mut().unwrap();
    mem::take(&mut exhaustive_data.deps)
  }

  fn register_exhaustive_callbacks(
    &mut self,
    once: bool,
    handler: Rc<dyn Fn(&mut Self) + 'a>,
    deps: FxHashSet<(ScopeId, SymbolId)>,
  ) where
    Self: EcmaAnalyzer<'a>,
  {
    for (scope, symbol) in deps {
      self
        .scoping_mut()
        .variable
        .get_mut(scope)
        .exhaustive_callbacks
        .entry(symbol)
        .or_default()
        .insert(ExhaustiveCallback { handler: handler.clone(), once });
    }
  }

  fn mark_exhaustive_read(&mut self, variable: (ScopeId, SymbolId), target: usize)
  where
    Self: EcmaAnalyzer<'a>,
  {
    for depth in target..self.scoping().cf.stack.len() {
      self.scoping_mut().cf.get_mut_from_depth(depth).mark_exhaustive_read(variable);
    }
  }

  fn mark_exhaustive_write(&mut self, variable: (ScopeId, SymbolId), target: usize) -> (bool, bool)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let mut should_consume = false;
    let mut indeterminate = false;
    for depth in target..self.scoping().cf.stack.len() {
      let scope = self.scoping_mut().cf.get_mut_from_depth(depth);
      if !should_consume {
        should_consume |= scope.mark_exhaustive_write(variable);
      }
      indeterminate |= scope.is_indeterminate();
    }
    (should_consume, indeterminate)
  }

  fn add_exhaustive_callbacks(
    &mut self,
    should_consume: bool,
    (scope, symbol): (ScopeId, SymbolId),
  ) -> bool
  where
    Self: EcmaAnalyzer<'a>,
  {
    if let Some(runners) =
      self.scoping_mut().variable.get_mut(scope).exhaustive_callbacks.get_mut(&symbol)
    {
      if runners.is_empty() {
        false
      } else {
        if should_consume {
          self.scoping_mut().pending_deps.extend(runners.drain());
        } else {
          self.scoping_mut().pending_deps.extend(runners.iter().cloned());
        }
        true
      }
    } else {
      false
    }
  }

  fn call_exhaustive_callbacks(&mut self) -> bool
  where
    Self: EcmaAnalyzer<'a>,
  {
    if self.scoping().pending_deps.is_empty() {
      return false;
    }
    loop {
      let runners = mem::take(&mut self.scoping_mut().pending_deps);
      for runner in runners {
        // let old_count = self.referred_deps.debug_count();
        let ExhaustiveCallback { handler: runner, once } = runner;
        let deps = self.exec_exhaustively("dep", runner.clone(), once);
        self.register_exhaustive_callbacks(once, runner, deps);
        // let new_count = self.referred_deps.debug_count();
        // self.debug += 1;
      }
      if self.scoping().pending_deps.is_empty() {
        return true;
      }
    }
  }

  fn has_exhaustive_scope_since(&self, target_depth: usize) -> bool
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping().cf.iter_stack_range(target_depth..).any(|scope| scope.is_exhaustive())
  }
}
