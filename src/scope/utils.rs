use crate::{
  analyzer::Analyzer,
  consumable::{ConsumableNode, ConsumableTrait},
};
use oxc::semantic::{ScopeId, SymbolId};
use std::mem;

impl<'a> Analyzer<'a> {
  pub fn find_first_different_cf_scope(&self, another: ScopeId) -> usize {
    self.scope_context.cf.find_lca(another).0 + 1
  }

  /// Returns (has_exhaustive, indeterminate, exec_deps)
  pub fn pre_possible_mutate(
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

  /// Returns (has_exhaustive, indeterminate, exec_deps)
  pub fn pre_must_mutate(
    &mut self,
    cf_scope: ScopeId,
    object_id: SymbolId,
  ) -> (bool, bool, ConsumableNode<'a, impl ConsumableTrait<'a> + 'a>) {
    let target_depth = self.find_first_different_cf_scope(cf_scope);

    let mut has_exhaustive = false;
    let mut indeterminate = false;
    let mut exec_deps = vec![];
    for depth in target_depth..self.scope_context.cf.stack.len() {
      let scope = self.scope_context.cf.get_mut_from_depth(depth);
      has_exhaustive |=
        scope.mark_exhaustive_write((self.scope_context.object_scope_id, object_id));
      indeterminate |= scope.is_indeterminate();
      if let Some(dep) = scope.deps.try_collect() {
        exec_deps.push(dep);
      }
    }
    self.exec_exhaustive_deps(true, (self.scope_context.object_scope_id, object_id));
    (has_exhaustive, indeterminate, ConsumableNode::new(exec_deps))
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
    self.exec_exhaustive_deps(true, (self.scope_context.object_scope_id, object_id));
  }

  pub fn mark_object_property_exhaustive_read(&mut self, cf_scope: ScopeId, object_id: SymbolId) {
    let target_depth = self.find_first_different_cf_scope(cf_scope);
    self.mark_exhaustive_read((self.scope_context.object_scope_id, object_id), target_depth);
  }

  pub fn mark_object_consumed(&mut self, cf_scope: ScopeId, object_id: SymbolId) {
    let target_depth = self.find_first_different_cf_scope(cf_scope);
    for depth in target_depth..self.scope_context.cf.stack.len() {
      let scope = self.scope_context.cf.get_mut_from_depth(depth);
      scope.mark_exhaustive_write((self.scope_context.object_scope_id, object_id));
      mem::take(&mut scope.deps).consume_all(self);
    }
    self.exec_exhaustive_deps(true, (self.scope_context.object_scope_id, object_id));
  }
}
