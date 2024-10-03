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
    let cf_scope_0 = cf.push(CfScope::new(CfScopeKind::Function, None, None, Some(false)));
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
    let cf_scope_depth = {
      self.scope_context.cf.push(CfScope::new(
        CfScopeKind::Function,
        None,
        Some(call_dep),
        Some(false),
      ));
      self.scope_context.cf.current_depth()
    };
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
    self.scope_context.cf.push(CfScope::new(kind, labels, None, exited));
    self.scope_context.cf.current_depth()
  }

  pub fn push_cf_scope_normal(&mut self, exited: Option<bool>) {
    self.push_cf_scope(CfScopeKind::Normal, None, exited);
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

  pub fn exit_to(&mut self, target_index: usize) {
    let mut must_exit = true;
    for (index, id) in self.scope_context.cf.stack.clone().into_iter().enumerate().rev() {
      let cf_scope = self.scope_context.cf.get_mut(id);
      if must_exit {
        let is_indeterminate = cf_scope.is_indeterminate();
        cf_scope.exited = Some(true);

        // Stop exiting outer scopes if one inner scope is indeterminate.
        if is_indeterminate {
          must_exit = false;
          if cf_scope.is_conditional() {
            // For the `if` statement, do not mark the outer scopes as indeterminate here.
            // Instead, let the `if` statement handle it.
            debug_assert!(cf_scope.blocked_exit.is_none());
            cf_scope.blocked_exit = Some(target_index);
            break;
          }
        }
      } else {
        cf_scope.exited = None;
      }
      if index == target_index {
        break;
      }
    }
  }

  /// If the label is used, `true` is returned.
  pub fn break_to_label(&mut self, label: Option<&'a str>) -> bool {
    let mut is_closest_breakable = true;
    let mut target_index = None;
    let mut label_used = false;
    for (idx, cf_scope) in self.scope_context.cf.iter_stack().enumerate().rev() {
      if cf_scope.is_function() {
        break;
      }
      let breakable_without_label = cf_scope.is_breakable_without_label();
      if let Some(label) = label {
        if let Some(label_entity) = cf_scope.matches_label(label) {
          if !is_closest_breakable || !breakable_without_label {
            self.referred_nodes.insert(label_entity.dep_node());
            label_used = true;
          }
          target_index = Some(idx);
          break;
        }
        if breakable_without_label {
          is_closest_breakable = false;
        }
      } else if breakable_without_label {
        target_index = Some(idx);
        break;
      }
    }
    self.exit_to(target_index.unwrap());
    label_used
  }

  /// If the label is used, `true` is returned.
  pub fn continue_to_label(&mut self, label: Option<&'a str>) -> bool {
    let mut is_closest_continuable = true;
    let mut target_index = None;
    let mut label_used = false;
    for (idx, cf_scope) in self.scope_context.cf.iter_stack().enumerate().rev() {
      if cf_scope.is_function() {
        break;
      }
      let is_continuable = cf_scope.is_continuable();
      if let Some(label) = label {
        if is_continuable {
          if let Some(label_entity) = cf_scope.matches_label(label) {
            if !is_closest_continuable {
              self.referred_nodes.insert(label_entity.dep_node());
              label_used = true;
            }
            target_index = Some(idx);
            break;
          }
          is_closest_continuable = false;
        }
      } else if is_continuable {
        target_index = Some(idx);
        break;
      }
    }
    self.exit_to(target_index.unwrap());
    label_used
  }
}
