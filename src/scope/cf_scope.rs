use oxc::semantic::ScopeId;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::entity::label::LabelEntity;

#[derive(Debug)]
/// `None` for indeterminate
/// `Some(true)` for exited
pub(crate) struct CfScope<'a> {
  pub label: Vec<LabelEntity<'a>>,
  pub id: ScopeId,
  pub exited: Option<bool>,
  pub is_loop_or_switch: bool,
}

static CF_SCOPE_ID: AtomicU32 = AtomicU32::new(0);

impl<'a> CfScope<'a> {
  pub(crate) fn new(
    label: Vec<LabelEntity<'a>>,
    exited: Option<bool>,
    is_loop_or_switch: bool,
  ) -> Self {
    CfScope {
      label,
      id: ScopeId::new(CF_SCOPE_ID.fetch_add(1, Ordering::Relaxed)),
      exited,
      is_loop_or_switch,
    }
  }

  pub(crate) fn must_exited(&self) -> bool {
    matches!(self.exited, Some(true))
  }

  pub(crate) fn is_indeterminate(&self) -> bool {
    self.exited.is_none()
  }

  pub(crate) fn matches_label(&self, label: &str) -> Option<&LabelEntity<'a>> {
    self.label.iter().find(|l| l.name == label)
  }
}
