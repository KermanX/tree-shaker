use crate::{
  analyzer::Analyzer,
  consumable::{
    box_consumable, Consumable, ConsumableCollector, ConsumableNode, ConsumableTrait, ConsumableVec,
  },
  entity::LabelEntity,
  utils::{DebuggerEvent, Logger},
};
use oxc::semantic::{ScopeId, SymbolId};
use rustc_hash::FxHashSet;
use std::{mem, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CfScopeKind {
  Indeterminate,
  Labeled,
  Dependent,
  BreakableWithoutLabel,
  Continuable,
  Exhaustive,
  IfBranch,
  ConditionalExprBranch,
  LogicalRight,
  Function,
  Block,
  Module,
}

#[derive(Debug)]
pub struct ExhaustiveData {
  pub dirty: bool,
  pub deps: FxHashSet<(ScopeId, SymbolId)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferredState {
  Never,
  ReferredClean,
  ReferredDirty,
}

#[derive(Debug)]
pub struct CfScope<'a> {
  pub kind: CfScopeKind,
  pub labels: Option<Rc<Vec<LabelEntity<'a>>>>,
  pub deps: ConsumableCollector<'a>,
  pub referred_state: ReferredState,
  pub exited: Option<bool>,
  /// Exits that have been stopped by this scope's indeterminate state.
  /// Only available when `kind` is `If`.
  pub blocked_exit: Option<usize>,
  pub exhaustive_data: Option<Box<ExhaustiveData>>,
}

impl<'a> CfScope<'a> {
  pub fn new(
    kind: CfScopeKind,
    labels: Option<Rc<Vec<LabelEntity<'a>>>>,
    deps: ConsumableVec<'a>,
    exited: Option<bool>,
  ) -> Self {
    CfScope {
      kind,
      labels,
      deps: ConsumableCollector::new(deps),
      referred_state: ReferredState::Never,
      exited,
      blocked_exit: None,
      exhaustive_data: if kind == CfScopeKind::Exhaustive {
        Some(Box::new(ExhaustiveData { dirty: true, deps: FxHashSet::default() }))
      } else {
        None
      },
    }
  }

  pub fn push_dep(&mut self, dep: Consumable<'a>) {
    self.deps.push(dep);
    if self.referred_state == ReferredState::ReferredClean {
      self.referred_state = ReferredState::ReferredDirty;
    }
  }

  pub fn update_exited(
    &mut self,
    id: ScopeId,
    logger: &Option<&Logger>,
    exited: Option<bool>,
    get_dep: impl FnOnce() -> Option<Consumable<'a>>,
  ) {
    if self.exited != Some(true) {
      self.exited = exited;
      if let Some(dep) = get_dep() {
        self.push_dep(dep);
      }

      if let Some(logger) = logger {
        logger.push_event(DebuggerEvent::UpdateCfScopeExited(id, exited));
      }
    }
  }

  pub fn must_exited(&self) -> bool {
    matches!(self.exited, Some(true))
  }

  pub fn is_indeterminate(&self) -> bool {
    self.exited.is_none()
  }

  pub fn matches_label(&self, label: &str) -> Option<&LabelEntity<'a>> {
    if let Some(labels) = &self.labels {
      labels.iter().find(|l| l.name == label)
    } else {
      None
    }
  }

  pub fn is_breakable_without_label(&self) -> bool {
    self.kind == CfScopeKind::BreakableWithoutLabel
  }

  pub fn is_continuable(&self) -> bool {
    self.kind == CfScopeKind::Continuable
  }

  pub fn is_if_branch(&self) -> bool {
    self.kind == CfScopeKind::IfBranch
  }

  pub fn is_function(&self) -> bool {
    self.kind == CfScopeKind::Function
  }

  pub fn is_exhaustive(&self) -> bool {
    self.kind == CfScopeKind::Exhaustive
  }

  pub fn mark_exhaustive_read(&mut self, variable: (ScopeId, SymbolId)) {
    if let Some(data) = &mut self.exhaustive_data {
      if !data.dirty {
        data.deps.insert(variable);
      }
    }
  }

  pub fn mark_exhaustive_write(&mut self, variable: (ScopeId, SymbolId)) -> bool {
    if let Some(data) = &mut self.exhaustive_data {
      if !data.dirty && data.deps.contains(&variable) {
        data.dirty = true;
      }
      true
    } else {
      false
    }
  }

  pub fn iterate_exhaustively(&mut self) -> bool {
    let exited = self.must_exited();
    let data = self.exhaustive_data.as_mut().unwrap();
    let dirty = data.dirty;
    data.dirty = false;
    if dirty && !exited {
      data.deps.clear();
      true
    } else {
      false
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn exec_indeterminately<T>(&mut self, runner: impl FnOnce(&mut Analyzer<'a>) -> T) -> T {
    self.push_indeterminate_cf_scope();
    let result = runner(self);
    self.pop_cf_scope();
    result
  }

  pub fn get_exec_dep(
    &mut self,
    target_depth: usize,
  ) -> ConsumableNode<'a, impl ConsumableTrait<'a> + 'a> {
    let mut deps = vec![];
    for id in target_depth..self.scope_context.cf.stack.len() {
      let scope = self.scope_context.cf.get_mut_from_depth(id);
      if let Some(dep) = scope.deps.try_collect() {
        deps.push(dep);
      }
    }
    ConsumableNode::new(deps)
  }

  pub fn exit_to(&mut self, target_depth: usize) {
    self.exit_to_impl(target_depth, self.scope_context.cf.stack.len(), true, None);
  }

  pub fn exit_to_not_must(&mut self, target_depth: usize) {
    self.exit_to_impl(target_depth, self.scope_context.cf.stack.len(), false, None);
  }

  /// `None` => Interrupted by if branch
  /// `Some` => Accumulated dependencies, may be `None`
  pub fn exit_to_impl(
    &mut self,
    target_depth: usize,
    from_depth: usize,
    mut must_exit: bool,
    mut acc_dep: Option<ConsumableNode<'a>>,
  ) -> Option<Option<ConsumableNode<'a>>> {
    for depth in (target_depth..from_depth).rev() {
      let id = self.scope_context.cf.stack[depth];
      let cf_scope = self.scope_context.cf.get_mut(id);
      let this_dep = cf_scope.deps.try_collect();
      let get_dep = || acc_dep.clone().map(box_consumable);

      // Update exited state
      if must_exit {
        let is_indeterminate = cf_scope.is_indeterminate();
        cf_scope.update_exited(id, &self.logger, Some(true), get_dep);

        // Stop exiting outer scopes if one inner scope is indeterminate.
        if is_indeterminate {
          must_exit = false;
          if cf_scope.is_if_branch() {
            // For the `if` statement, do not mark the outer scopes as indeterminate here.
            // Instead, let the `if` statement handle it.
            debug_assert!(cf_scope.blocked_exit.is_none());
            cf_scope.blocked_exit = Some(target_depth);
            return None;
          }
        }
      } else {
        cf_scope.update_exited(id, &self.logger, None, get_dep);
      }

      // Accumulate the dependencies
      if let Some(this_dep) = this_dep.clone() {
        acc_dep = if let Some(acc_dep) = acc_dep {
          Some(ConsumableNode::new_box((this_dep, acc_dep)))
        } else {
          Some(this_dep)
        };
      }
    }
    Some(acc_dep)
  }

  /// If the label is used, `true` is returned.
  pub fn break_to_label(&mut self, label: Option<&'a str>) -> bool {
    let mut is_closest_breakable = true;
    let mut target_depth = None;
    let mut label_used = false;
    for (idx, cf_scope) in self.scope_context.cf.iter_stack().enumerate().rev() {
      if cf_scope.is_function() {
        break;
      }
      let breakable_without_label = cf_scope.is_breakable_without_label();
      if let Some(label) = label {
        if let Some(label_entity) = cf_scope.matches_label(label) {
          if !is_closest_breakable || !breakable_without_label {
            self.referred_deps.refer_dep(label_entity.dep_id());
            label_used = true;
          }
          target_depth = Some(idx);
          break;
        }
        if breakable_without_label {
          is_closest_breakable = false;
        }
      } else if breakable_without_label {
        target_depth = Some(idx);
        break;
      }
    }
    self.exit_to(target_depth.unwrap());
    label_used
  }

  /// If the label is used, `true` is returned.
  pub fn continue_to_label(&mut self, label: Option<&'a str>) -> bool {
    let mut is_closest_continuable = true;
    let mut target_depth = None;
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
              self.referred_deps.refer_dep(label_entity.dep_id());
              label_used = true;
            }
            target_depth = Some(idx);
            break;
          }
          is_closest_continuable = false;
        }
      } else if is_continuable {
        target_depth = Some(idx);
        break;
      }
    }
    self.exit_to(target_depth.unwrap());
    label_used
  }

  pub fn refer_to_global(&mut self) {
    if self.is_inside_pure() {
      return;
    }

    for depth in (0..self.scope_context.cf.stack.len()).rev() {
      let scope = self.scope_context.cf.get_mut_from_depth(depth);
      match scope.referred_state {
        ReferredState::Never => {
          scope.referred_state = ReferredState::ReferredClean;
          mem::take(&mut scope.deps).consume_all(self);
        }
        ReferredState::ReferredClean => break,
        ReferredState::ReferredDirty => {
          scope.referred_state = ReferredState::ReferredClean;
          mem::take(&mut scope.deps).consume_all(self);
          for depth in (0..depth).rev() {
            let scope = self.scope_context.cf.get_mut_from_depth(depth);
            match scope.referred_state {
              ReferredState::Never => unreachable!("Logic error in refer_to_global"),
              ReferredState::ReferredClean => break,
              ReferredState::ReferredDirty => {
                scope.deps.force_clear();
                scope.referred_state = ReferredState::ReferredClean;
              }
            }
          }
          break;
        }
      }
    }
  }
}
