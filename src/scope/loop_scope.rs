use oxc::{ast::ast::LabelIdentifier, semantic::ScopeId};

#[derive(Debug)]
pub(crate) struct LoopScope<'a> {
  pub label: Option<&'a str>,
  pub cf_scope_id: ScopeId,
}

impl<'a> LoopScope<'a> {
  pub(crate) fn new(label: Option<&'a LabelIdentifier<'a>>, cf_scope_id: ScopeId) -> Self {
    LoopScope {
      label: label.map(|label| label.name.as_str()),
      cf_scope_id,
    }
  }
}
