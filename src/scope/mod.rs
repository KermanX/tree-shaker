mod function_scope;
mod loop_scope;
mod variable_scope;
use crate::analyzer::Analyzer;
use function_scope::FunctionScope;
use loop_scope::LoopScope;
use variable_scope::VariableScope;

#[derive(Debug, Default)]
pub(crate) struct ScopeContext<'a> {
  pub function_scopes: Vec<FunctionScope<'a>>,
  pub loop_scopes: Vec<LoopScope<'a>>,
  pub variable_scopes: Vec<VariableScope<'a>>,
  pub indeterminate_scopes: Vec<bool>,
}

impl<'a> ScopeContext<'a> {
  pub(crate) fn new() -> Self {
    ScopeContext {
      function_scopes: vec![FunctionScope::new()],
      loop_scopes: vec![],
      variable_scopes: vec![VariableScope::new()],
      indeterminate_scopes: vec![false],
    }
  }
}

impl<'a> Analyzer<'a> {
  pub(crate) fn function_scope(&self) -> &FunctionScope<'a> {
    self.scope_context.function_scopes.last().unwrap()
  }

  pub(crate) fn loop_scope(&self) -> &LoopScope<'a> {
    self.scope_context.loop_scopes.last().unwrap()
  }

  pub(crate) fn variable_scope(&self) -> &VariableScope<'a> {
    self.scope_context.variable_scopes.last().unwrap()
  }

  pub(crate) fn indeterminate_scope(&self) -> bool {
    *self.scope_context.indeterminate_scopes.last().unwrap()
  }

  pub(crate) fn function_scope_mut(&mut self) -> &mut FunctionScope<'a> {
    self.scope_context.function_scopes.last_mut().unwrap()
  }

  pub(crate) fn loop_scope_mut(&mut self) -> &mut LoopScope<'a> {
    self.scope_context.loop_scopes.last_mut().unwrap()
  }

  pub(crate) fn variable_scope_mut(&mut self) -> &mut VariableScope<'a> {
    self.scope_context.variable_scopes.last_mut().unwrap()
  }

  pub(crate) fn set_indeterminate_scope(&mut self, new_value: bool) {
    *self.scope_context.indeterminate_scopes.last_mut().unwrap() = new_value;
  }

  pub(crate) fn push_function_scope(&mut self) {
    self.scope_context.function_scopes.push(FunctionScope::new());
  }

  pub(crate) fn pop_function_scope(&mut self) -> FunctionScope<'a> {
    self.scope_context.function_scopes.pop().unwrap()
  }

  pub(crate) fn push_variable_scope(&mut self) {
    self.scope_context.variable_scopes.push(VariableScope::new());
  }

  pub(crate) fn pop_variable_scope(&mut self) -> VariableScope<'a> {
    self.scope_context.variable_scopes.pop().unwrap()
  }

  pub(crate) fn push_indeterminate_scope(&mut self, indeterminate: bool) {
    self.scope_context.indeterminate_scopes.push(indeterminate);
  }

  pub(crate) fn pop_indeterminate_scope(&mut self) -> bool {
    self.scope_context.indeterminate_scopes.pop().unwrap()
  }
}
