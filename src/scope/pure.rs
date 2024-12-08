use crate::{analyzer::Analyzer, dep::ReferredDeps};
use std::mem;

impl<'a> Analyzer<'a> {
  pub fn exec_in_pure<T>(
    &mut self,
    runner: impl FnOnce(&mut Analyzer<'a>) -> T,
    referred_deps: &'a mut ReferredDeps,
  ) -> (T, &'a mut ReferredDeps) {
    let parent_referred_deps = mem::replace(&mut self.referred_deps, referred_deps);
    let val = runner(self);
    let this_referred_deps = mem::replace(&mut self.referred_deps, parent_referred_deps);
    (val, this_referred_deps)
  }
}
