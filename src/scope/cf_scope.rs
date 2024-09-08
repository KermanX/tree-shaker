use oxc::semantic::ScopeId;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::entity::label::LabelEntity;

#[derive(Debug)]
/// `None` for indeterminate
/// `Some(true)` for exited
pub struct CfScope<'a> {
  pub label: Vec<LabelEntity<'a>>,
  pub id: ScopeId,
  pub exited: Option<bool>,
  pub is_loop_or_switch: bool,
  // Exits that have been stopped by this scope's indeterminate state.
  pub stopped_exit: Option<usize>,
}

static CF_SCOPE_ID: AtomicU32 = AtomicU32::new(0);

impl<'a> CfScope<'a> {
  pub fn new(label: Vec<LabelEntity<'a>>, exited: Option<bool>, is_loop_or_switch: bool) -> Self {
    CfScope {
      label,
      id: ScopeId::new(CF_SCOPE_ID.fetch_add(1, Ordering::Relaxed)),
      exited,
      is_loop_or_switch,
      stopped_exit: None,
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
}
