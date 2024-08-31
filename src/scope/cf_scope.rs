use oxc::semantic::ScopeId;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone)]
/// `None` for indeterminate
/// `Some(true)` for exited
pub(crate) struct CfScope<'a> {
  pub label: Vec<&'a str>,
  pub id: ScopeId,
  pub exited: Option<bool>,
}

static CF_SCOPE_ID: AtomicU32 = AtomicU32::new(0);

impl<'a> CfScope<'a> {
  pub(crate) fn new(label: Vec<&'a str>, exited: Option<bool>) -> Self {
    CfScope { label, id: ScopeId::new(CF_SCOPE_ID.fetch_add(1, Ordering::Relaxed)), exited }
  }

  pub(crate) fn must_exited(&self) -> bool {
    matches!(self.exited, Some(true))
  }

  pub(crate) fn is_indeterminate(&self) -> bool {
    self.exited.is_none()
  }
}
