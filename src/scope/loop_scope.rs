use oxc::semantic::ScopeId;

#[derive(Debug)]
pub(crate) struct LoopScope<'a> {
  pub label: Option<&'a str>,
  pub cf_scope_id: ScopeId,
}

impl<'a> LoopScope<'a> {
  pub(crate) fn new(label: Option<&'a str>, cf_scope_id: ScopeId) -> Self {
    LoopScope { label, cf_scope_id }
  }
}
