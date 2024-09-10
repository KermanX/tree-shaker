use oxc::semantic::SymbolId;
use rustc_hash::FxHashSet;

use crate::entity::label::LabelEntity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CfScopeKind {
  Normal,
  Breakable,
  Exhaustive,
  If,
}

#[derive(Debug)]
pub struct ExhaustiveData {
  pub dirty: bool,
  pub deps: FxHashSet<SymbolId>,
}

#[derive(Debug)]
/// `None` for indeterminate
/// `Some(true)` for exited
pub struct CfScope<'a> {
  pub kind: CfScopeKind,
  pub label: Vec<LabelEntity<'a>>,
  pub exited: Option<bool>,
  // Exits that have been stopped by this scope's indeterminate state.
  // Only available when `kind` is `If`.
  pub stopped_exit: Option<usize>,
  pub exhaustive_data: Option<Box<ExhaustiveData>>,
}

impl<'a> CfScope<'a> {
  pub fn new(kind: CfScopeKind, label: Vec<LabelEntity<'a>>, exited: Option<bool>) -> Self {
    CfScope {
      kind,
      label,
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
    self.label.iter().find(|l| l.name == label)
  }

  pub fn is_breakable(&self) -> bool {
    matches!(self.kind, CfScopeKind::Breakable)
  }

  pub fn is_if(&self) -> bool {
    matches!(self.kind, CfScopeKind::If)
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

  pub fn check_and_clear_exhaustive_dirty(&mut self) -> bool {
    if let Some(data) = &mut self.exhaustive_data {
      let dirty = data.dirty;
      data.dirty = false;
      data.deps.clear();
      dirty
    } else {
      unreachable!()
    }
  }
}
