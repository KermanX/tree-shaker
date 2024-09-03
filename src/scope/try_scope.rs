use crate::entity::{entity::Entity, unknown::UnknownEntity};
use oxc::semantic::ScopeId;

#[derive(Debug)]
pub struct TryScope<'a> {
  pub thrown_values: Vec<Entity<'a>>,
  pub cf_scope_id: ScopeId,
}

impl<'a> TryScope<'a> {
  pub fn new(cf_scope_id: ScopeId) -> Self {
    TryScope { thrown_values: Vec::new(), cf_scope_id }
  }

  pub fn thrown_val(self) -> Entity<'a> {
    // Always unknown here
    UnknownEntity::new_unknown_with_deps(self.thrown_values)
  }
}
