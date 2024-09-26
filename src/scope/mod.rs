pub mod call_scope;
pub mod cf_scope;
pub mod exhaustive;
pub mod try_scope;
mod utils;
pub mod variable_scope;

use crate::{
  analyzer::Analyzer,
  entity::{Consumable, Entity, EntityDepNode, LabelEntity, UnknownEntity},
};
use call_scope::CallScope;
pub use cf_scope::CfScopeKind;
use cf_scope::{CfScope, CfScopes};
use oxc::semantic::SymbolId;
use std::{borrow::Borrow, cell::RefCell, mem, rc::Rc};
use try_scope::TryScope;
use variable_scope::{VariableScope, VariableScopes};

#[derive(Debug, Default)]
pub struct ScopeContext<'a> {
  pub call_scopes: Vec<CallScope<'a>>,
  pub variable_scopes: VariableScopes<'a>,
  pub cf_scopes: CfScopes<'a>,
}

impl<'a> ScopeContext<'a> {
  pub fn new() -> Self {
    let cf_scopes =
      vec![Rc::new(RefCell::new(CfScope::new(CfScopeKind::Function, None, Some(false))))];
    ScopeContext {
      call_scopes: vec![CallScope::new(
        EntityDepNode::Environment,
        ().into(),
        vec![],
        0,
        0,
        // TODO: global this
        UnknownEntity::new_unknown(),
        (UnknownEntity::new_unknown(), vec![]),
        true,
        false,
      )],
      variable_scopes: vec![Rc::new(RefCell::new(VariableScope::new(None, cf_scopes.clone())))],
      cf_scopes,
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn call_scope(&self) -> &CallScope<'a> {
    self.scope_context.call_scopes.last().unwrap()
  }

  pub fn variable_scope(&mut self) -> &Rc<RefCell<VariableScope<'a>>> {
    self.scope_context.variable_scopes.last().unwrap()
  }

  pub fn cf_scope(&self) -> &Rc<RefCell<CfScope<'a>>> {
    self.scope_context.cf_scopes.last().unwrap()
  }

  pub fn call_scope_mut(&mut self) -> &mut CallScope<'a> {
    self.scope_context.call_scopes.last_mut().unwrap()
  }

  pub fn push_call_scope(
    &mut self,
    source: impl Into<EntityDepNode>,
    call_dep: impl Into<Consumable<'a>>,
    variable_scopes: Rc<VariableScopes<'a>>,
    this: Entity<'a>,
    args: (Entity<'a>, Vec<SymbolId>),
    is_async: bool,
    is_generator: bool,
  ) {
    let call_dep = call_dep.into();
    let mut call_stack_deps: Vec<_> =
      self.scope_context.call_scopes.iter().map(|scope| scope.borrow().call_dep.clone()).collect();
    call_stack_deps.push(call_dep.clone());

    let old_variable_scopes =
      mem::replace(&mut self.scope_context.variable_scopes, variable_scopes.as_ref().clone());
    let variable_scope_index = self.scope_context.variable_scopes.len();
    self.scope_context.variable_scopes.push(Rc::new(RefCell::new(VariableScope::new(
      Some(call_stack_deps.into()),
      self.scope_context.cf_scopes.clone(),
    ))));
    let cf_scope_index = self.push_cf_scope(CfScopeKind::Function, None, Some(false));
    self.scope_context.call_scopes.push(CallScope::new(
      source.into(),
      call_dep,
      old_variable_scopes,
      cf_scope_index,
      variable_scope_index,
      this,
      args,
      is_async,
      is_generator,
    ));
  }

  pub fn pop_call_scope(&mut self) -> Entity<'a> {
    let scope = self.scope_context.call_scopes.pop().unwrap();
    let (old_variable_scopes, ret_val) = scope.finalize(self);
    self.pop_cf_scope();
    self.pop_variable_scope();
    self.scope_context.variable_scopes = old_variable_scopes;
    ret_val
  }

  pub fn push_variable_scope(&mut self) {
    self
      .scope_context
      .variable_scopes
      .push(Rc::new(RefCell::new(VariableScope::new(None, self.scope_context.cf_scopes.clone()))));
  }

  pub fn pop_variable_scope(&mut self) {
    self.scope_context.variable_scopes.pop().unwrap();
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
    let index = self.scope_context.cf_scopes.len();
    let cf_scope = Rc::new(RefCell::new(CfScope::new(kind, labels, exited)));
    self.scope_context.cf_scopes.push(cf_scope);
    index
  }

  pub fn push_cf_scope_normal(&mut self, exited: Option<bool>) {
    self.push_cf_scope(CfScopeKind::Normal, None, exited);
  }

  pub fn pop_cf_scope(&mut self) -> Rc<RefCell<CfScope<'a>>> {
    self.scope_context.cf_scopes.pop().unwrap()
  }

  pub fn try_scope(&self) -> &TryScope<'a> {
    self.call_scope().try_scopes.last().unwrap()
  }

  pub fn try_scope_mut(&mut self) -> &mut TryScope<'a> {
    self.call_scope_mut().try_scopes.last_mut().unwrap()
  }

  pub fn push_try_scope(&mut self) {
    let cf_scope_index = self.push_cf_scope(CfScopeKind::Normal, None, None);
    self.call_scope_mut().try_scopes.push(TryScope::new(cf_scope_index));
  }

  pub fn pop_try_scope(&mut self) -> TryScope<'a> {
    self.pop_cf_scope();
    self.call_scope_mut().try_scopes.pop().unwrap()
  }

  pub fn exit_to(&mut self, target_index: usize) {
    let mut must_exit = true;
    for (idx, cf_scope) in self.scope_context.cf_scopes.iter().enumerate().rev() {
      let mut cf_scope = cf_scope.borrow_mut();
      if must_exit {
        let is_indeterminate = cf_scope.is_indeterminate();
        cf_scope.exited = Some(true);

        // Stop exiting outer scopes if one inner scope is indeterminate.
        if is_indeterminate {
          must_exit = false;
          if cf_scope.is_if() {
            // For the `if` statement, do not mark the outer scopes as indeterminate here.
            // Instead, let the `if` statement handle it.
            debug_assert!(cf_scope.stopped_exit.is_none());
            cf_scope.stopped_exit = Some(target_index);
            break;
          }
        }
      } else {
        cf_scope.exited = None;
      }
      if idx == target_index {
        break;
      }
    }
  }

  /// If the label is used, `true` is returned.
  pub fn break_to_label(&mut self, label: Option<&'a str>) -> bool {
    let mut is_closest_breakable = true;
    let mut target_index = None;
    let mut label_used = false;
    for (idx, cf_scope) in self.scope_context.cf_scopes.iter().enumerate().rev() {
      let cf_scope = cf_scope.as_ref().borrow();
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
    for (idx, cf_scope) in self.scope_context.cf_scopes.iter().enumerate().rev() {
      let cf_scope = cf_scope.as_ref().borrow();
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
