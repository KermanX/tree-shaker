pub mod call_scope;
pub mod cf_scope;
pub mod conditional;
pub mod exhaustive;
mod scope_tree;
pub mod try_scope;
mod utils;
pub mod variable_scope;

use crate::{
  analyzer::Analyzer,
  entity::{Consumable, Entity, EntityDepNode, LabelEntity, UnknownEntity},
};
use call_scope::CallScope;
use cf_scope::CfScope;
pub use cf_scope::CfScopeKind;
use oxc::semantic::{ScopeId, SymbolId};
use scope_tree::ScopeTree;
use std::{mem, rc::Rc};
use try_scope::TryScope;
use variable_scope::VariableScope;

pub struct ScopeContext<'a> {
  pub call: Vec<CallScope<'a>>,
  pub variable: ScopeTree<VariableScope<'a>>,
  pub cf: ScopeTree<CfScope<'a>>,
}

impl<'a> ScopeContext<'a> {
  pub fn new() -> Self {
    let mut cf = ScopeTree::new();
    let cf_scope_0 = cf.push(CfScope::new(CfScopeKind::Module, None, vec![], Some(false)));
    let mut variable = ScopeTree::new();
    let body_variable_scope = variable.push(VariableScope::new(cf_scope_0, 0));
    ScopeContext {
      call: vec![CallScope::new(
        EntityDepNode::Environment,
        vec![],
        0,
        0,
        body_variable_scope,
        // TODO: global this
        UnknownEntity::new_unknown(),
        (UnknownEntity::new_unknown(), vec![]),
        true,
        false,
      )],
      variable,
      cf,
    }
  }

  pub fn assert_final_state(&self) {
    debug_assert_eq!(self.call.len(), 1);
    debug_assert_eq!(self.variable.current_depth(), 0);
    debug_assert_eq!(self.cf.current_depth(), 0);
  }
}

impl<'a> Analyzer<'a> {
  pub fn call_scope(&self) -> &CallScope<'a> {
    self.scope_context.call.last().unwrap()
  }

  pub fn call_scope_mut(&mut self) -> &mut CallScope<'a> {
    self.scope_context.call.last_mut().unwrap()
  }

  pub fn try_scope(&self) -> &TryScope<'a> {
    self.call_scope().try_scopes.last().unwrap()
  }

  pub fn try_scope_mut(&mut self) -> &mut TryScope<'a> {
    self.call_scope_mut().try_scopes.last_mut().unwrap()
  }

  pub fn variable_scope(&mut self) -> &VariableScope<'a> {
    self.scope_context.variable.get_current()
  }

  pub fn cf_scope(&self) -> &CfScope<'a> {
    self.scope_context.cf.get_current()
  }

  pub fn cf_scope_mut(&mut self) -> &mut CfScope<'a> {
    self.scope_context.cf.get_current_mut()
  }

  pub fn push_call_scope(
    &mut self,
    source: impl Into<EntityDepNode>,
    call_dep: impl Into<Consumable<'a>>,
    variable_scope_stack: Rc<Vec<ScopeId>>,
    this: Entity<'a>,
    args: (Entity<'a>, Vec<SymbolId>),
    is_async: bool,
    is_generator: bool,
  ) {
    let call_dep = call_dep.into();

    // FIXME: no clone
    let variable_scope_stack = variable_scope_stack.as_ref().clone();
    let old_variable_scope_stack = self.scope_context.variable.replace_stack(variable_scope_stack);
    let body_variable_scope = self.push_variable_scope();
    let variable_scope_depth = self.scope_context.variable.current_depth();
    let cf_scope_depth =
      self.push_cf_scope_with_deps(CfScopeKind::Function, None, vec![call_dep], Some(false));
    self.scope_context.call.push(CallScope::new(
      source.into(),
      old_variable_scope_stack,
      cf_scope_depth,
      variable_scope_depth,
      body_variable_scope,
      this,
      args,
      is_async,
      is_generator,
    ));
  }

  pub fn pop_call_scope(&mut self) -> Entity<'a> {
    let scope = self.scope_context.call.pop().unwrap();
    let (old_variable_scope_stack, ret_val) = scope.finalize(self);
    self.pop_cf_scope();
    self.pop_variable_scope();
    self.scope_context.variable.replace_stack(old_variable_scope_stack);
    ret_val
  }

  pub fn push_variable_scope(&mut self) -> ScopeId {
    self.scope_context.variable.push(VariableScope::new(
      self.scope_context.cf.current_id(),
      self.scope_context.cf.current_depth(),
    ))
  }

  pub fn pop_variable_scope(&mut self) -> ScopeId {
    self.scope_context.variable.pop()
  }

  pub fn push_exec_dep(&mut self, dep: impl Into<Consumable<'a>>) {
    // self.call_scope_mut().exec_deps.push(dep.into());
  }

  pub fn pop_exec_dep(&mut self) {
    // self.call_scope_mut().exec_deps.pop();
  }

  pub fn take_labels(&mut self) -> Option<Rc<Vec<LabelEntity<'a>>>> {
    if self.pending_labels.is_empty() {
      None
    } else {
      Some(Rc::new(mem::take(&mut self.pending_labels)))
    }
  }

  pub fn push_cf_scope(
    &mut self,
    kind: CfScopeKind,
    labels: Option<Rc<Vec<LabelEntity<'a>>>>,
    exited: Option<bool>,
  ) -> usize {
    self.push_cf_scope_with_deps(kind, labels, vec![], exited)
  }

  pub fn push_cf_scope_with_deps(
    &mut self,
    kind: CfScopeKind,
    labels: Option<Rc<Vec<LabelEntity<'a>>>>,
    deps: Vec<Consumable<'a>>,
    exited: Option<bool>,
  ) -> usize {
    self.scope_context.cf.push(CfScope::new(kind, labels, deps, exited));
    self.scope_context.cf.current_depth()
  }

  pub fn push_cf_scope_normal(&mut self, exited: Option<bool>) {
    self.push_cf_scope(CfScopeKind::Normal, None, exited);
  }

  pub fn push_cf_scope_for_deps(&mut self, deps: Vec<Consumable<'a>>) {
    self.push_cf_scope_with_deps(CfScopeKind::Normal, None, deps, Some(false));
  }

  pub fn pop_cf_scope(&mut self) -> ScopeId {
    self.scope_context.cf.pop()
  }

  pub fn pop_cf_scope_and_get(&mut self) -> &CfScope<'a> {
    let id = self.pop_cf_scope();
    self.scope_context.cf.get(id)
  }

  pub fn push_try_scope(&mut self) {
    let cf_scope_index = self.scope_context.cf.current_depth() - 1;
    self.push_cf_scope(CfScopeKind::Normal, None, None);
    let variable_scope_index = self.scope_context.call.len() - 1;
    self.call_scope_mut().try_scopes.push(TryScope::new(cf_scope_index, variable_scope_index));
  }

  pub fn pop_try_scope(&mut self) -> TryScope<'a> {
    self.pop_cf_scope();
    self.call_scope_mut().try_scopes.pop().unwrap()
  }
}
