use super::{cf_scope, loop_scope::LoopScope};
use crate::entity::{entity::Entity, literal::LiteralEntity, union::UnionEntity};
use oxc::semantic::ScopeId;

#[derive(Debug)]
pub(crate) struct FunctionScope<'a> {
  /// `None` for indeterminate
  pub returned_value: Vec<Entity<'a>>,
  pub loop_scopes: Vec<LoopScope<'a>>,
  pub cf_scope_id: ScopeId,
}

impl<'a> FunctionScope<'a> {
  pub(crate) fn new(cf_scope_id: ScopeId) -> Self {
    FunctionScope { returned_value: Vec::new(), loop_scopes: Vec::new(), cf_scope_id }
  }

  pub(crate) fn ret_val(self) -> Entity<'a> {
    if self.returned_value.is_empty() {
      LiteralEntity::new_undefined()
    } else {
      UnionEntity::new(self.returned_value)
    }
  }
}
