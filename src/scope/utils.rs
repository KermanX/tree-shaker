use super::cf_scope::CfScope;
use crate::{analyzer::Analyzer, entity::Consumable};
use oxc::semantic::ScopeId;

impl<'a> Analyzer<'a> {
  pub fn find_first_different_cf_scope(&self, another: ScopeId) -> usize {
    self.scope_context.cf.find_lca(another).0 + 1
  }

  pub fn find_first_different_variable_scope(&self, another: ScopeId) -> usize {
    self.scope_context.variable.find_lca(another).0 + 1
  }

  pub fn is_relatively_indeterminate(&self, target_cf_scope: usize) -> bool {
    self.scope_context.cf.iter_stack_range(target_cf_scope..).any(CfScope::is_indeterminate)
  }

  pub fn is_assignment_indeterminate(&self, another: ScopeId) -> bool {
    let first_different = self.find_first_different_cf_scope(another);
    self.is_relatively_indeterminate(first_different)
  }

  pub fn get_assignment_deps(
    &self,
    target_depth: usize,
    extra: impl Into<Consumable<'a>>,
  ) -> Consumable<'a> {
    if target_depth == 0 {
      self.get_exec_dep(0, extra)
    } else {
      let variable_scope = self.scope_context.variable.get_from_depth(target_depth - 1).unwrap();
      self.get_exec_dep(variable_scope.cf_scope_depth + 1, extra)
    }
  }
}
