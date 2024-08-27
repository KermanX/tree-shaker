mod cf_scope;
mod function_scope;
mod loop_scope;
mod variable_scope;

use crate::analyzer::Analyzer;
use cf_scope::CfScope;
use function_scope::FunctionScope;
use loop_scope::LoopScope;
use oxc::semantic::ScopeId;
use variable_scope::VariableScope;

#[derive(Debug, Default)]
pub(crate) struct ScopeContext<'a> {
  pub function_scopes: Vec<FunctionScope<'a>>,
  pub variable_scopes: Vec<VariableScope<'a>>,
  pub cf_scopes: Vec<CfScope>,
}

impl<'a> ScopeContext<'a> {
  pub(crate) fn new() -> Self {
    let cf_scope_0 = CfScope::new(Some(false));
    ScopeContext {
      function_scopes: vec![FunctionScope::new(cf_scope_0.id)],
      variable_scopes: vec![VariableScope::new()],
      cf_scopes: vec![cf_scope_0],
    }
  }
}

impl<'a> Analyzer<'a> {
  pub(crate) fn function_scope(&self) -> &FunctionScope<'a> {
    self.scope_context.function_scopes.last().unwrap()
  }

  pub(crate) fn loop_scope(&self) -> &LoopScope<'a> {
    self.function_scope().loop_scopes.last().unwrap()
  }

  pub(crate) fn loop_scope_by_label(&self, label: Option<&'a str>) -> &LoopScope<'a> {
    if let Some(label) = label {
      for scope in self.function_scope().loop_scopes.iter().rev() {
        if let Some(scope_label) = scope.label {
          if scope_label == label {
            return scope;
          }
        }
      }
      unreachable!();
    } else {
      self.loop_scope()
    }
  }

  pub(crate) fn variable_scope(&self) -> &VariableScope<'a> {
    self.scope_context.variable_scopes.last().unwrap()
  }

  pub(crate) fn cf_scope(&self) -> CfScope {
    *self.scope_context.cf_scopes.last().unwrap()
  }

  pub(crate) fn function_scope_mut(&mut self) -> &mut FunctionScope<'a> {
    self.scope_context.function_scopes.last_mut().unwrap()
  }

  pub(crate) fn loop_scope_mut(&mut self) -> &mut LoopScope<'a> {
    self.function_scope_mut().loop_scopes.last_mut().unwrap()
  }

  pub(crate) fn variable_scope_mut(&mut self) -> &mut VariableScope<'a> {
    self.scope_context.variable_scopes.last_mut().unwrap()
  }

  pub(crate) fn push_function_scope(&mut self) {
    let cf_scope_id = self.push_cf_scope(Some(false));
    self.scope_context.function_scopes.push(FunctionScope::new(cf_scope_id));
  }

  pub(crate) fn pop_function_scope(&mut self) -> FunctionScope<'a> {
    self.pop_cf_scope();
    let scope = self.scope_context.function_scopes.pop().unwrap();
    debug_assert!(scope.loop_scopes.is_empty());
    scope
  }

  pub(crate) fn push_loop_scope(&mut self) {
    let label = self.current_label;
    let cf_scope_id = self.push_cf_scope(Some(false));
    self.function_scope_mut().loop_scopes.push(LoopScope::new(label, cf_scope_id));
  }

  pub(crate) fn pop_loop_scope(&mut self) -> LoopScope<'a> {
    self.pop_cf_scope();
    self.function_scope_mut().loop_scopes.pop().unwrap()
  }

  pub(crate) fn push_variable_scope(&mut self) {
    self.scope_context.variable_scopes.push(VariableScope::new());
  }

  pub(crate) fn pop_variable_scope(&mut self) -> VariableScope<'a> {
    self.scope_context.variable_scopes.pop().unwrap()
  }

  pub(crate) fn push_cf_scope(&mut self, exited: Option<bool>) -> ScopeId {
    let cf_scope = CfScope::new(exited);
    let id = cf_scope.id;
    self.scope_context.cf_scopes.push(cf_scope);
    id
  }

  pub(crate) fn pop_cf_scope(&mut self) -> CfScope {
    self.scope_context.cf_scopes.pop().unwrap()
  }

  pub(crate) fn exit_to(&mut self, cf_scope_id: ScopeId) {
    for cf_scope in self.scope_context.cf_scopes.iter_mut().rev() {
      let is_indeterminate = cf_scope.is_indeterminate();
      cf_scope.exited = Some(true);
      if cf_scope.id == cf_scope_id || is_indeterminate {
        break;
      }
    }
  }
}
