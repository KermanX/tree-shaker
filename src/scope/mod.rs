mod function_scope;
mod loop_scope;

use function_scope::FunctionScope;
use loop_scope::LoopScope;

#[derive(Debug, Default)]
pub(crate) struct ScopeContext<'a> {
  function_scopes: Vec<FunctionScope<'a>>,
  loop_scopes: Vec<LoopScope<'a>>,
}

impl<'a> ScopeContext<'a> {
  pub(crate) fn new() -> Self {
    ScopeContext { function_scopes: Vec::new(), loop_scopes: Vec::new() }
  }

  pub(crate) fn function_scope(&mut self) -> &mut FunctionScope<'a> {
    self.function_scopes.last_mut().unwrap()
  }

  pub(crate) fn loop_scope(&mut self) -> &mut LoopScope<'a> {
    self.loop_scopes.last_mut().unwrap()
  }
}
