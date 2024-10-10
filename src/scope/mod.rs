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
  consumable::{box_consumable, Consumable, ConsumableTrait, ConsumableVec},
  entity::{Entity, EntityFactory, FunctionEntitySource, LabelEntity},
  logger::DebuggerEvent,
};
use call_scope::CallScope;
use cf_scope::CfScope;
pub use cf_scope::CfScopeKind;
use oxc::{
  semantic::{ScopeId, SymbolId},
  span::GetSpan,
};
use scope_tree::ScopeTree;
use std::rc::Rc;
use try_scope::TryScope;
use variable_scope::VariableScope;

pub struct ScopeContext<'a> {
  pub call: Vec<CallScope<'a>>,
  pub variable: ScopeTree<VariableScope<'a>>,
  pub cf: ScopeTree<CfScope<'a>>,
}

impl<'a> ScopeContext<'a> {
  pub fn new(factory: &EntityFactory<'a>) -> Self {
    let mut cf = ScopeTree::new();
    let cf_scope_0 = cf.push(CfScope::new(CfScopeKind::Module, None, vec![], Some(false)));
    let mut variable = ScopeTree::new();
    let body_variable_scope = variable.push(VariableScope::new(cf_scope_0));
    ScopeContext {
      call: vec![CallScope::new(
        FunctionEntitySource::Module,
        vec![],
        0,
        body_variable_scope,
        // TODO: global this
        factory.unknown,
        (factory.unknown, vec![]),
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

  pub fn cf_scope(&self) -> &CfScope<'a> {
    self.scope_context.cf.get_current()
  }

  pub fn cf_scope_mut(&mut self) -> &mut CfScope<'a> {
    self.scope_context.cf.get_current_mut()
  }

  fn replace_variable_scope_stack(&mut self, new_stack: Vec<ScopeId>) -> Vec<ScopeId> {
    if let Some(logger) = self.logger {
      logger.push_event(DebuggerEvent::ReplaceVarScopeStack(new_stack.clone()));
    }

    self.scope_context.variable.replace_stack(new_stack)
  }

  pub fn push_call_scope(
    &mut self,
    source: FunctionEntitySource<'a>,
    call_dep: Consumable<'a>,
    variable_scope_stack: Rc<Vec<ScopeId>>,
    this: Entity<'a>,
    args: (Entity<'a>, Vec<SymbolId>),
    is_async: bool,
    is_generator: bool,
  ) {
    // FIXME: no clone
    let variable_scope_stack = variable_scope_stack.as_ref().clone();
    let old_variable_scope_stack = self.replace_variable_scope_stack(variable_scope_stack);
    let body_variable_scope = self.push_variable_scope();
    let cf_scope_depth =
      self.push_cf_scope_with_dep(CfScopeKind::Function, None, vec![call_dep], Some(false));

    if let Some(logger) = self.logger {
      logger.push_event(DebuggerEvent::PushCallScope(
        source.span(),
        old_variable_scope_stack.clone(),
        cf_scope_depth,
        body_variable_scope,
      ));
    }

    self.scope_context.call.push(CallScope::new(
      source.into(),
      old_variable_scope_stack,
      cf_scope_depth,
      body_variable_scope,
      this,
      args,
      is_async,
      is_generator,
    ));
  }

  pub fn pop_call_scope(&mut self) -> Entity<'a> {
    let scope = self.scope_context.call.pop().unwrap();

    if let Some(logger) = self.logger {
      logger.push_event(DebuggerEvent::PopCallScope);
    }

    let (old_variable_scope_stack, ret_val) = scope.finalize(self);
    self.pop_cf_scope();
    self.pop_variable_scope();
    self.replace_variable_scope_stack(old_variable_scope_stack);
    ret_val
  }

  pub fn push_variable_scope(&mut self) -> ScopeId {
    let id =
      self.scope_context.variable.push(VariableScope::new(self.scope_context.cf.current_id()));

    if let Some(logger) = self.logger {
      logger.push_event(DebuggerEvent::PushVarScope(id, self.scope_context.cf.current_id()));
    }

    id
  }

  pub fn pop_variable_scope(&mut self) -> ScopeId {
    if let Some(logger) = self.logger {
      logger.push_event(DebuggerEvent::PopVarScope);
    }

    self.scope_context.variable.pop()
  }

  pub fn push_cf_scope(
    &mut self,
    kind: CfScopeKind,
    labels: Option<Rc<Vec<LabelEntity<'a>>>>,
    exited: Option<bool>,
  ) -> usize {
    self.push_cf_scope_with_dep(kind, labels, vec![], exited)
  }

  pub fn push_cf_scope_with_dep(
    &mut self,
    kind: CfScopeKind,
    labels: Option<Rc<Vec<LabelEntity<'a>>>>,
    deps: ConsumableVec<'a>,
    exited: Option<bool>,
  ) -> usize {
    self.scope_context.cf.push(CfScope::new(kind, labels, deps, exited));

    if let Some(logger) = self.logger {
      logger.push_event(DebuggerEvent::PushCfScope(
        self.scope_context.cf.current_id(),
        kind,
        exited,
      ));
    }

    self.scope_context.cf.current_depth()
  }

  pub fn push_cf_scope_normal(&mut self, exited: Option<bool>) {
    self.push_cf_scope(CfScopeKind::Normal, None, exited);
  }

  pub fn push_cf_scope_for_dep(&mut self, dep: impl ConsumableTrait<'a> + 'a) {
    self.push_cf_scope_with_dep(CfScopeKind::Normal, None, vec![box_consumable(dep)], Some(false));
  }

  pub fn pop_cf_scope(&mut self) -> ScopeId {
    if let Some(logger) = self.logger {
      logger.push_event(DebuggerEvent::PopCfScope);
    }

    self.scope_context.cf.pop()
  }

  pub fn pop_cf_scope_and_get_mut(&mut self) -> &mut CfScope<'a> {
    let id = self.pop_cf_scope();
    self.scope_context.cf.get_mut(id)
  }

  pub fn push_try_scope(&mut self) {
    self.push_cf_scope(CfScopeKind::Normal, None, None);
    let cf_scope_depth = self.scope_context.cf.current_depth();
    self.call_scope_mut().try_scopes.push(TryScope::new(cf_scope_depth));
  }

  pub fn pop_try_scope(&mut self) -> TryScope<'a> {
    self.pop_cf_scope();
    self.call_scope_mut().try_scopes.pop().unwrap()
  }
}
