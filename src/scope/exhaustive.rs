use crate::{analyzer::Analyzer, entity::entity::Entity, scope::CfScopeKind};
use oxc::semantic::SymbolId;

impl<'a> Analyzer<'a> {
  pub fn exec_exhaustively(&mut self, runner: impl Fn(&mut Analyzer<'a>) -> ()) {
    self.push_cf_scope(CfScopeKind::Exhaustive, Some(false));

    let mut round_counter = 0;

    while self.cf_scope_mut().check_and_clear_exhaustive_dirty() {
      runner(self);
      round_counter += 1;
      if round_counter > 1000 {
        unreachable!("Exhaustive loop is too long");
      }
    }

    self.pop_cf_scope();
  }

  pub fn mark_exhaustive_read(&mut self, val: &Entity<'a>, symbol: SymbolId, target: usize) {
    if !val.test_is_completely_unknown() {
      for scope in &mut self.scope_context.cf_scopes[target..] {
        scope.mark_exhaustive_read(symbol)
      }
    }
  }

  pub fn mark_exhaustive_write(
    &mut self,
    old_val: &Entity<'a>,
    symbol: SymbolId,
    target: usize,
  ) -> bool {
    if old_val.test_is_completely_unknown() {
      false
    } else {
      let mut should_consume = false;
      for scope in &mut self.scope_context.cf_scopes[target..] {
        should_consume |= scope.mark_exhaustive_write(symbol)
      }
      should_consume
    }
  }
}
