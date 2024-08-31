use crate::entity::{entity::Entity, literal::LiteralEntity, union::UnionEntity};
use oxc::semantic::ScopeId;

#[derive(Debug)]
pub(crate) struct FunctionScope<'a> {
  /// `None` for indeterminate
  pub returned_value: Vec<Entity<'a>>,
  pub cf_scope_id: ScopeId,
  pub this: Entity<'a>,
}

impl<'a> FunctionScope<'a> {
  pub(crate) fn new(cf_scope_id: ScopeId, this: Entity<'a>) -> Self {
    FunctionScope { returned_value: Vec::new(), cf_scope_id, this }
  }

  pub(crate) fn ret_val(self) -> Entity<'a> {
    if self.returned_value.is_empty() {
      LiteralEntity::new_undefined()
    } else {
      UnionEntity::new(self.returned_value)
    }
  }
}
