use crate::entity::{
  entity::Entity, literal::LiteralEntity, promise::PromiseEntity, union::UnionEntity,
};
use oxc::semantic::ScopeId;

#[derive(Debug)]
pub struct FunctionScope<'a> {
  /// `None` for indeterminate
  pub returned_value: Vec<Entity<'a>>,
  pub cf_scope_id: ScopeId,
  pub this: Entity<'a>,
  pub is_async: bool,
  pub has_await_effect: bool,
}

impl<'a> FunctionScope<'a> {
  pub fn new(cf_scope_id: ScopeId, this: Entity<'a>, is_async: bool) -> Self {
    FunctionScope {
      returned_value: Vec::new(),
      cf_scope_id,
      this,
      is_async,
      has_await_effect: false,
    }
  }

  pub fn ret_val(self) -> Entity<'a> {
    let value = if self.returned_value.is_empty() {
      LiteralEntity::new_undefined()
    } else {
      UnionEntity::new(self.returned_value)
    };
    if self.is_async {
      PromiseEntity::new(self.has_await_effect, value)
    } else {
      value
    }
  }
}
