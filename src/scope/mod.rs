mod cf_scope;
mod function_scope;
mod try_scope;
mod variable_scope;

use crate::{
  analyzer::Analyzer,
  entity::{entity::Entity, unknown::UnknownEntity},
};
use cf_scope::CfScope;
use function_scope::FunctionScope;
use oxc::semantic::ScopeId;
use std::mem;
use try_scope::TryScope;
use variable_scope::VariableScope;

#[derive(Debug, Default)]
pub struct ScopeContext<'a> {
  pub function_scopes: Vec<FunctionScope<'a>>,
  pub variable_scopes: Vec<VariableScope<'a>>,
  pub cf_scopes: Vec<CfScope<'a>>,
}

impl<'a> ScopeContext<'a> {
  pub fn new() -> Self {
    let cf_scope_0 = CfScope::new(vec![], Some(false), false);
    ScopeContext {
      function_scopes: vec![FunctionScope::new(
        cf_scope_0.id,
        // TODO: global this
        UnknownEntity::new_unknown(),
        true,
        false,
      )],
      variable_scopes: vec![VariableScope::new(cf_scope_0.id)],
      cf_scopes: vec![cf_scope_0],
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn function_scope(&self) -> &FunctionScope<'a> {
    self.scope_context.function_scopes.last().unwrap()
  }

  pub fn variable_scope(&self) -> &VariableScope<'a> {
    self.scope_context.variable_scopes.last().unwrap()
  }

  pub fn cf_scope(&self) -> &CfScope<'a> {
    self.scope_context.cf_scopes.last().unwrap()
  }

  pub fn function_scope_mut(&mut self) -> &mut FunctionScope<'a> {
    self.scope_context.function_scopes.last_mut().unwrap()
  }

  pub fn variable_scope_mut(&mut self) -> &mut VariableScope<'a> {
    self.scope_context.variable_scopes.last_mut().unwrap()
  }

  pub fn cf_scope_mut(&mut self) -> &mut CfScope<'a> {
    self.scope_context.cf_scopes.last_mut().unwrap()
  }

  pub fn push_function_scope(&mut self, this: Entity<'a>, is_async: bool, is_generator: bool) {
    let cf_scope_id = self.push_cf_scope(Some(false), false);
    self.push_variable_scope(cf_scope_id);
    self.scope_context.function_scopes.push(FunctionScope::new(
      cf_scope_id,
      this,
      is_async,
      is_generator,
    ));
  }

  pub fn pop_function_scope(&mut self) -> (bool, Entity<'a>) {
    let ret_val = self.scope_context.function_scopes.pop().unwrap().ret_val(self);
    let has_effect = self.pop_variable_scope().has_effect;
    self.pop_cf_scope();
    (has_effect, ret_val)
  }

  pub fn push_variable_scope(&mut self, cf_scope_id: ScopeId) {
    self.scope_context.variable_scopes.push(VariableScope::new(cf_scope_id));
  }

  pub fn pop_variable_scope(&mut self) -> VariableScope<'a> {
    self.scope_context.variable_scopes.pop().unwrap()
  }

  pub fn variable_scope_path(&self) -> Vec<ScopeId> {
    self.scope_context.variable_scopes.iter().map(|x| x.id).collect()
  }

  pub fn get_variable_scope_by_id(&self, id: ScopeId) -> &VariableScope<'a> {
    self.scope_context.variable_scopes.iter().find(|x| x.id == id).unwrap()
  }

  pub fn get_variable_scope_by_id_mut(&mut self, id: ScopeId) -> &mut VariableScope<'a> {
    self.scope_context.variable_scopes.iter_mut().find(|x| x.id == id).unwrap()
  }

  pub fn push_cf_scope(&mut self, exited: Option<bool>, is_loop_or_switch: bool) -> ScopeId {
    let label = mem::take(&mut self.pending_labels);
    let cf_scope = CfScope::new(label, exited, is_loop_or_switch);
    let id = cf_scope.id;
    self.scope_context.cf_scopes.push(cf_scope);
    id
  }

  pub fn pop_cf_scope(&mut self) -> CfScope {
    self.scope_context.cf_scopes.pop().unwrap()
  }

  pub fn try_scope(&self) -> &TryScope<'a> {
    self.function_scope().try_scopes.last().unwrap()
  }

  pub fn try_scope_mut(&mut self) -> &mut TryScope<'a> {
    self.function_scope_mut().try_scopes.last_mut().unwrap()
  }

  pub fn push_try_scope(&mut self) {
    let cf_scope_id = self.push_cf_scope(Some(false), false);
    self.function_scope_mut().try_scopes.push(TryScope::new(cf_scope_id));
  }

  pub fn pop_try_scope(&mut self) -> TryScope<'a> {
    self.pop_cf_scope();
    self.function_scope_mut().try_scopes.pop().unwrap()
  }

  pub fn exit_to(&mut self, cf_scope_id: ScopeId) {
    for cf_scope in self.scope_context.cf_scopes.iter_mut().rev() {
      let is_indeterminate = cf_scope.is_indeterminate();
      cf_scope.exited = Some(true);
      if cf_scope.id == cf_scope_id || is_indeterminate {
        break;
      }
    }
  }

  /// If the label is used, `true` is returned.
  pub fn exit_to_label(&mut self, label: Option<&'a str>) -> bool {
    let mut is_closest = true;
    let mut should_exit = true;
    for cf_scope in self.scope_context.cf_scopes.iter_mut().rev() {
      if should_exit {
        // Stop exiting outer scopes if one inner scope is indeterminate.
        should_exit = !cf_scope.is_indeterminate();

        cf_scope.exited = Some(true);
      } else {
        cf_scope.exited = None;
      }
      if let Some(label) = label {
        if let Some(label_entity) = cf_scope.matches_label(&label) {
          return if !is_closest || !cf_scope.is_loop_or_switch {
            self.referred_nodes.insert(label_entity.node);
            true
          } else {
            false
          };
        }
      } else if cf_scope.is_loop_or_switch {
        return false;
      }
      if cf_scope.is_loop_or_switch {
        is_closest = false;
      }
    }
    unreachable!();
  }

  pub fn is_relative_indeterminate(&self, target: ScopeId) -> bool {
    for cf_scope in self.scope_context.cf_scopes.iter().rev() {
      if cf_scope.is_indeterminate() {
        return true;
      }
      if cf_scope.id == target {
        return false;
      }
    }
    unreachable!();
  }
}
