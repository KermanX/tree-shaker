use crate::{analyzer::Analyzer, dep::ReferredDeps, entity::Entity};
use std::mem;

impl<'a> Analyzer<'a> {
  pub fn exec_in_pure(
    &mut self,
    runner: impl FnOnce(&mut Analyzer<'a>) -> Entity<'a>,
  ) -> (Entity<'a>, ReferredDeps) {
    let parent_referred_deps = mem::replace(&mut self.referred_deps, ReferredDeps::default());
    let val = runner(self);
    let this_referred_deps = mem::replace(&mut self.referred_deps, parent_referred_deps);
    (val, this_referred_deps)
  }
}
