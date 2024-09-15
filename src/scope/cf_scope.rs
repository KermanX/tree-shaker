use crate::entity::label::LabelEntity;
use oxc::semantic::SymbolId;
use rustc_hash::FxHashSet;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CfScopeKind {
  Normal,
  BreakableWithoutLabel,
  Continuable,
  Exhaustive,
  If,
  Function,
}

#[derive(Debug)]
pub struct ExhaustiveData {
  pub dirty: bool,
  pub deps: FxHashSet<SymbolId>,
}

#[derive(Debug)]
pub struct CfScope<'a> {
  pub kind: CfScopeKind,
  pub labels: Option<Rc<Vec<LabelEntity<'a>>>>,
  pub exited: Option<bool>,
  // Exits that have been stopped by this scope's indeterminate state.
  // Only available when `kind` is `If`.
  pub stopped_exit: Option<usize>,
  pub exhaustive_data: Option<Box<ExhaustiveData>>,
}

pub type CfScopes<'a> = Vec<Rc<RefCell<CfScope<'a>>>>;

impl<'a> CfScope<'a> {
  pub fn new(
    kind: CfScopeKind,
    labels: Option<Rc<Vec<LabelEntity<'a>>>>,
    exited: Option<bool>,
  ) -> Self {
    CfScope {
      kind,
      labels,
      exited,
      stopped_exit: None,
      exhaustive_data: if kind == CfScopeKind::Exhaustive {
        Some(Box::new(ExhaustiveData { dirty: true, deps: FxHashSet::default() }))
      } else {
        None
      },
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

  pub fn is_if(&self) -> bool {
    self.kind == CfScopeKind::If
  }

  pub fn is_function(&self) -> bool {
    self.kind == CfScopeKind::Function
  }

  pub fn mark_exhaustive_read(&mut self, symbol: SymbolId) {
    if let Some(data) = &mut self.exhaustive_data {
      if !data.dirty {
        data.deps.insert(symbol);
      }
    }
  }

  pub fn mark_exhaustive_write(&mut self, symbol: SymbolId) -> bool {
    if let Some(data) = &mut self.exhaustive_data {
      if !data.dirty && data.deps.contains(&symbol) {
        data.dirty = true;
      }
      true
    } else {
      false
    }
  }

  pub fn clear_exhaustive_dirty(&mut self) {
    if let Some(data) = &mut self.exhaustive_data {
      data.dirty = false;
    } else {
      unreachable!()
    }
  }

  pub fn iterate_exhaustively(&mut self) -> bool {
    if let Some(data) = &mut self.exhaustive_data {
      let dirty = data.dirty;
      data.dirty = false;
      data.deps.clear();
      dirty && !self.must_exited()
    } else {
      unreachable!()
    }
  }
}
