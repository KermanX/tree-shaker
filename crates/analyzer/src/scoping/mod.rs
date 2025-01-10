pub mod call_scope;
pub mod cf_scope;
pub mod exhaustive;
mod scope_tree;
pub mod try_scope;
pub mod variable_scope;

pub use call_scope::*;
pub use cf_scope::*;
pub use exhaustive::*;
use oxc::{ast::ast::LabeledStatement, semantic::ScopeId};
use oxc_index::Idx;
use rustc_hash::FxHashSet;
use scope_tree::ScopeTree;
use std::rc::Rc;
pub use try_scope::*;
pub use variable_scope::*;

use crate::EcmaAnalyzer;

pub struct Scoping<'a, A: EcmaAnalyzer<'a> + ?Sized> {
  pub labels: Vec<&'a LabeledStatement<'a>>,
  pub call: Vec<CallScope<'a, A>>,
  pub variable: ScopeTree<VariableScope<'a, A>>,
  pub cf: ScopeTree<CfScope<'a, A>>,

  pub pending_deps: FxHashSet<ExhaustiveCallback<'a, A>>,

  pub object_scope_id: ScopeId,
  pub object_symbol_counter: usize,
}

impl<'a, A: EcmaAnalyzer<'a> + ?Sized> Default for Scoping<'a, A> {
  fn default() -> Self {
    Scoping {
      labels: vec![],
      call: vec![],
      variable: ScopeTree::new(),
      cf: ScopeTree::new(),

      pending_deps: FxHashSet::default(),

      object_scope_id: ScopeId::from_usize(0),
      object_symbol_counter: 128,
    }
  }
}

// impl<'a> Scoping<'a> {
//   pub fn new(factory: &EntityFactory<'a>) -> Self {
//     let mut cf = ScopeTree::new();
//     cf.push(CfScope::new(CfScopeKind::Module, None, vec![], Some(false)));
//     let mut variable = ScopeTree::new();
//     let body_variable_scope = variable.push({
//       let mut scope = VariableScope::new();
//       scope.this = Some(factory.unknown());
//       scope
//     });
//     let object_scope_id = variable.add_special(VariableScope::new());
//     Scoping {
//       call: vec![CallScope::new(
//         DepId::from_counter(),
//         CalleeInfo {
//           node: CalleeNode::Module,
//           instance_id: factory.alloc_instance_id(),
//           #[cfg(feature = "flame")]
//           debug_name: "<Module>",
//         },
//         vec![],
//         0,
//         body_variable_scope,
//         true,
//         false,
//       )],
//       variable,
//       cf,
//       pure: 0,

//       object_scope_id,
//       object_symbol_counter: 128,
//     }
//   }

//   pub fn assert_final_state(&mut self) {
//     assert_eq!(self.call.len(), 1);
//     assert_eq!(self.variable.current_depth(), 0);
//     assert_eq!(self.cf.current_depth(), 0);
//     assert_eq!(self.pure, 0);

//     for scope in self.cf.iter_all() {
//       if let Some(data) = &scope.exhaustive_data {
//         assert!(!data.dirty);
//       }
//     }

//     #[cfg(feature = "flame")]
//     self.call.pop().unwrap().scope_guard.end();
//   }

//   pub fn alloc_object_id(&mut self) -> SymbolId {
//     self.object_symbol_counter += 1;
//     SymbolId::from_usize(self.object_symbol_counter)
//   }
// }

pub trait ScopingAnalyzer<'a>:
  CallScopeAnalyzer<'a>
  + CfScopeAnalyzer<'a>
  + ExhaustiveScopeAnalyzer<'a>
  + LoopScopeAnalyzer<'a>
  + VariableScopeAnalyzer<'a>
  + TryScopeAnalyzer<'a>
{
  fn scoping(&self) -> &Scoping<'a, Self>
  where
    Self: EcmaAnalyzer<'a>;

  fn scoping_mut(&mut self) -> &mut Scoping<'a, Self>
  where
    Self: EcmaAnalyzer<'a>;

  fn init_scoping(&mut self)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let cf_0 = CfScope::new(CfScopeKind::Module, None, Some(false), Default::default());
    self.scoping_mut().cf.push(cf_0);

    let mut variable_0 = VariableScope::new();
    variable_0.this = Some(self.global_this());
    let body_variable_scope = self.scoping_mut().variable.push(variable_0);

    let object_scope_id = self.scoping_mut().variable.add_special(VariableScope::new());
    self.scoping_mut().object_scope_id = object_scope_id;

    let call_0 = CallScope::new(
      0,
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
    );
    self.scoping_mut().call.push(call_0);
  }

  fn call_scope(&self) -> &CallScope<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping().call.last().unwrap()
  }

  fn call_scope_mut(&mut self) -> &mut CallScope<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping_mut().call.last_mut().unwrap()
  }

  fn try_scope(&self) -> &TryScope<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.call_scope().try_scopes.last().unwrap()
  }

  fn try_scope_mut(&mut self) -> &mut TryScope<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.call_scope_mut().try_scopes.last_mut().unwrap()
  }

  fn cf_scope(&self) -> &CfScope<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping().cf.get_current()
  }

  fn cf_scope_mut(&mut self) -> &mut CfScope<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping_mut().cf.get_current_mut()
  }

  fn cf_scope_id_of_call_scope(&self) -> ScopeId
  where
    Self: EcmaAnalyzer<'a>,
  {
    let depth = self.call_scope().cf_scope_depth;
    self.scoping().cf.stack[depth]
  }

  fn variable_scope(&self) -> &VariableScope<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping().variable.get_current()
  }

  fn variable_scope_mut(&mut self) -> &mut VariableScope<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping_mut().variable.get_current_mut()
  }

  fn is_inside_pure(&self) -> bool
  where
    Self: EcmaAnalyzer<'a>,
  {
    // TODO: self.scope_context.pure > 0
    false
  }

  fn replace_variable_scope_stack(&mut self, new_stack: Vec<ScopeId>) -> Vec<ScopeId>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping_mut().variable.replace_stack(new_stack)
  }

  fn push_call_scope(
    &mut self,
    callee: CalleeInfo<'a>,
    call_dep: Consumable<'a>,
    variable_scope_stack: Vec<ScopeId>,
    is_async: bool,
    is_generator: bool,
    consume: bool,
  ) where
    Self: EcmaAnalyzer<'a>,
  {
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

    self.scoping_mut().call.push(CallScope::new(
      dep_id,
      callee,
      old_variable_scope_stack,
      cf_scope_depth,
      body_variable_scope,
      is_async,
      is_generator,
    ));
  }

  fn pop_call_scope(&mut self) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>,
  {
    let scope = self.scoping_mut().call.pop().unwrap();
    let (old_variable_scope_stack, ret_val) = scope.finalize(self);
    self.pop_cf_scope();
    self.pop_variable_scope();
    self.replace_variable_scope_stack(old_variable_scope_stack);
    ret_val
  }

  fn push_variable_scope(&mut self) -> ScopeId
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping_mut().variable.push(VariableScope::new())
  }

  fn pop_variable_scope(&mut self) -> ScopeId
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scope_context.variable.pop()
  }

  fn push_cf_scope(
    &mut self,
    kind: CfScopeKind,
    labels: Option<Rc<Vec<&'a LabeledStatement<'a>>>>,
    exited: Option<bool>,
  ) -> usize
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.push_cf_scope_with_deps(kind, labels, vec![], exited)
  }

  fn push_indeterminate_cf_scope(&mut self)
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.push_cf_scope(CfScopeKind::Indeterminate, None, None);
  }

  fn pop_cf_scope(&mut self) -> ScopeId
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.scoping_mut().cf.pop()
  }

  fn pop_multiple_cf_scopes(&mut self, count: usize) -> Vec<&Self::CfScopeExtra>
  where
    Self: EcmaAnalyzer<'a>,
  {
    let mut extras = vec![];
    for _ in 0..count {
      let id = self.scoping_mut().cf.stack.pop().unwrap();
      extras.push(&self.scoping_mut().cf.get_mut(id).extra);
    }
    extras
  }

  fn pop_cf_scope_and_get_mut(&mut self) -> &mut CfScope<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    let id = self.pop_cf_scope();
    self.scoping_mut().cf.get_mut(id)
  }

  fn push_try_scope(&mut self)
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.push_indeterminate_cf_scope();
    let cf_scope_depth = self.scoping_mut().cf.current_depth();
    self.call_scope_mut().try_scopes.push(TryScope::new(cf_scope_depth));
  }

  fn pop_try_scope(&mut self) -> TryScope<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.pop_cf_scope();
    self.call_scope_mut().try_scopes.pop().unwrap()
  }
}
