use crate::{
  analyzer::Analyzer,
  consumable::{ConsumableNode, ConsumableTrait},
};
use oxc::semantic::{ScopeId, SymbolId};

impl<'a> Analyzer<'a> {
  pub fn find_first_different_cf_scope(&self, another: ScopeId) -> usize {
    self.scope_context.cf.find_lca(another).0 + 1
  }

  pub fn find_first_different_variable_scope(&self, another: ScopeId) -> usize {
    self.scope_context.variable.find_lca(another).0 + 1
  }

  pub fn get_assignment_dep(
    &mut self,
    target_depth: usize,
  ) -> ConsumableNode<'a, impl ConsumableTrait<'a> + 'a> {
    if target_depth == 0 {
      self.get_exec_dep(0)
    } else {
      let variable_scope = self.scope_context.variable.get_from_depth(target_depth - 1);
      let target_cf_depth = self.find_first_different_cf_scope(variable_scope.cf_scope);
      self.get_exec_dep(target_cf_depth)
    }
  }

  pub fn refer_to_diff_cf_scope(&mut self, cf_scope: ScopeId) {
    let target_depth = self.find_first_different_cf_scope(cf_scope);
    let dep = self.get_exec_dep(target_depth);
    self.consume(dep);
  }

  /// Returns (has_exhaustive, indeterminate, exec_deps)
  pub fn pre_mutate_object(
    &mut self,
    target_depth: usize,
  ) -> (bool, bool, ConsumableNode<'a, impl ConsumableTrait<'a> + 'a>) {
    let mut has_exhaustive = false;
    let mut indeterminate = false;
    let mut exec_deps = vec![];
    for depth in target_depth..self.scope_context.cf.stack.len() {
      let scope = self.scope_context.cf.get_mut_from_depth(depth);
      has_exhaustive |= scope.is_exhaustive();
      indeterminate |= scope.is_indeterminate();
      if let Some(dep) = scope.deps.try_collect() {
        exec_deps.push(dep);
      }
    }
    (has_exhaustive, indeterminate, ConsumableNode::new(exec_deps))
  }

  /// Returns (indeterminate, exec_deps)
  pub fn pre_mutate_array(
    &mut self,
    cf_scope: ScopeId,
    object_id: SymbolId,
  ) -> (bool, ConsumableNode<'a, impl ConsumableTrait<'a> + 'a>) {
    let target_depth = self.find_first_different_cf_scope(cf_scope);

    let mut indeterminate = false;
    let mut exec_deps = vec![];
    for depth in target_depth..self.scope_context.cf.stack.len() {
      let scope = self.scope_context.cf.get_mut_from_depth(depth);
      scope.mark_exhaustive_write((self.scope_context.object_scope_id, object_id));
      indeterminate |= scope.is_indeterminate();
      if let Some(dep) = scope.deps.try_collect() {
        exec_deps.push(dep);
      }
    }
    (indeterminate, ConsumableNode::new(exec_deps))
  }

  pub fn mark_object_property_exhaustive_write(
    &mut self,
    target_depth: usize,
    object_id: SymbolId,
  ) {
    for depth in target_depth..self.scope_context.cf.stack.len() {
      let scope = self.scope_context.cf.get_mut_from_depth(depth);
      scope.mark_exhaustive_write((self.scope_context.object_scope_id, object_id));
    }
  }

  pub fn mark_object_property_exhaustive_read(&mut self, cf_scope: ScopeId, object_id: SymbolId) {
    let target_depth = self.find_first_different_cf_scope(cf_scope);
    self.mark_exhaustive_read((self.scope_context.object_scope_id, object_id), target_depth);
  }
}
