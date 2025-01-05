pub mod call_scope;
pub mod cf_scope;
pub mod conditional;
pub mod exhaustive;
pub mod r#loop;
mod scope_tree;
pub mod try_scope;
mod utils;
pub mod variable_scope;

use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableVec},
  dep::DepId,
  entity::{Entity, EntityFactory, LabelEntity},
  utils::{CalleeInfo, CalleeNode},
};
use call_scope::CallScope;
use cf_scope::CfScope;
pub use cf_scope::CfScopeKind;
use oxc::semantic::{ScopeId, SymbolId};
use oxc_index::Idx;
use scope_tree::ScopeTree;
use std::rc::Rc;
use try_scope::TryScope;
use variable_scope::VariableScope;

pub struct ScopeContext<'a> {
  pub call: Vec<CallScope<'a>>,
  pub variable: ScopeTree<VariableScope<'a>>,
  pub cf: ScopeTree<CfScope<'a>>,
  pub pure: usize,

  pub object_scope_id: ScopeId,
  pub object_symbol_counter: usize,
}

impl<'a> ScopeContext<'a> {
  pub fn new(factory: &EntityFactory<'a>) -> Self {
    let mut cf = ScopeTree::new();
    cf.push(CfScope::new(CfScopeKind::Module, None, vec![], Some(false)));
    let mut variable = ScopeTree::new();
    let body_variable_scope = variable.push({
      let mut scope = VariableScope::new();
      scope.this = Some(factory.unknown());
      scope
    });
    let object_scope_id = variable.add_special(VariableScope::new());
    ScopeContext {
      call: vec![CallScope::new(
        DepId::from_counter(),
        CalleeInfo {
          node: CalleeNode::Module,
          instance_id: factory.alloc_instance_id(),
          #[cfg(feature = "flame")]
          debug_name: "<Module>",
        },
        vec![],
        0,
        body_variable_scope,
        true,
        false,
      )],
      variable,
      cf,
      pure: 0,

      object_scope_id,
      object_symbol_counter: 128,
    }
  }

  pub fn assert_final_state(&mut self) {
    assert_eq!(self.call.len(), 1);
    assert_eq!(self.variable.current_depth(), 0);
    assert_eq!(self.cf.current_depth(), 0);
    assert_eq!(self.pure, 0);

    for scope in self.cf.iter_all() {
      if let Some(data) = &scope.exhaustive_data {
        assert!(!data.dirty);
      }
    }

    #[cfg(feature = "flame")]
    self.call.pop().unwrap().scope_guard.end();
  }

  pub fn alloc_object_id(&mut self) -> SymbolId {
    self.object_symbol_counter += 1;
    SymbolId::from_usize(self.object_symbol_counter)
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

  pub fn cf_scope_id_of_call_scope(&self) -> ScopeId {
    let depth = self.call_scope().cf_scope_depth;
    self.scope_context.cf.stack[depth]
  }

  pub fn variable_scope(&self) -> &VariableScope<'a> {
    self.scope_context.variable.get_current()
  }

  pub fn variable_scope_mut(&mut self) -> &mut VariableScope<'a> {
    self.scope_context.variable.get_current_mut()
  }

  pub fn is_inside_pure(&self) -> bool {
    // TODO: self.scope_context.pure > 0
    false
  }

  fn replace_variable_scope_stack(&mut self, new_stack: Vec<ScopeId>) -> Vec<ScopeId> {
    self.scope_context.variable.replace_stack(new_stack)
  }

  pub fn push_call_scope(
    &mut self,
    callee: CalleeInfo<'a>,
    call_dep: Consumable<'a>,
    variable_scope_stack: Vec<ScopeId>,
    is_async: bool,
    is_generator: bool,
    consume: bool,
  ) {
    let dep_id = DepId::from_counter();
    if consume {
      self.refer_dep(dep_id);
    }

    let old_variable_scope_stack = self.replace_variable_scope_stack(variable_scope_stack);
    let body_variable_scope = self.push_variable_scope();
    let cf_scope_depth = self.push_cf_scope_with_deps(
      CfScopeKind::Function,
      None,
      vec![call_dep, self.consumable(dep_id)],
      Some(false),
    );

    self.scope_context.call.push(CallScope::new(
      dep_id,
      callee,
      old_variable_scope_stack,
      cf_scope_depth,
      body_variable_scope,
      is_async,
      is_generator,
    ));
  }

  pub fn pop_call_scope(&mut self) -> Entity<'a> {
    let scope = self.scope_context.call.pop().unwrap();
    let (old_variable_scope_stack, ret_val) = scope.finalize(self);
    self.pop_cf_scope();
    self.pop_variable_scope();
    self.replace_variable_scope_stack(old_variable_scope_stack);
    ret_val
  }

  pub fn push_variable_scope(&mut self) -> ScopeId {
    self.scope_context.variable.push(VariableScope::new())
  }

  pub fn pop_variable_scope(&mut self) -> ScopeId {
    self.scope_context.variable.pop()
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
    deps: ConsumableVec<'a>,
    exited: Option<bool>,
  ) -> usize {
    self.scope_context.cf.push(CfScope::new(kind, labels, deps, exited));
    self.scope_context.cf.current_depth()
  }

  pub fn push_indeterminate_cf_scope(&mut self) {
    self.push_cf_scope(CfScopeKind::Indeterminate, None, None);
  }

  pub fn push_dependent_cf_scope(&mut self, dep: impl Into<Consumable<'a>>) {
    self.push_cf_scope_with_deps(CfScopeKind::Dependent, None, vec![dep.into()], Some(false));
  }

  pub fn pop_cf_scope(&mut self) -> ScopeId {
    self.scope_context.cf.pop()
  }

  pub fn pop_multiple_cf_scopes(&mut self, count: usize) -> Option<Consumable<'a>> {
    let mut exec_deps = vec![];
    for _ in 0..count {
      let id = self.scope_context.cf.stack.pop().unwrap();
      if let Some(dep) = self.scope_context.cf.get_mut(id).deps.try_collect(self.factory) {
        exec_deps.push(dep);
      }
    }
    if exec_deps.is_empty() {
      None
    } else {
      Some(self.consumable(exec_deps))
    }
  }

  pub fn pop_cf_scope_and_get_mut(&mut self) -> &mut CfScope<'a> {
    let id = self.pop_cf_scope();
    self.scope_context.cf.get_mut(id)
  }

  pub fn push_try_scope(&mut self) {
    self.push_indeterminate_cf_scope();
    let cf_scope_depth = self.scope_context.cf.current_depth();
    self.call_scope_mut().try_scopes.push(TryScope::new(cf_scope_depth));
  }

  pub fn pop_try_scope(&mut self) -> TryScope<'a> {
    self.pop_cf_scope();
    self.call_scope_mut().try_scopes.pop().unwrap()
  }
}
