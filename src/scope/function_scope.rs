use super::try_scope::TryScope;
use crate::{
  analyzer::Analyzer,
  entity::{entity::Entity, literal::LiteralEntity, promise::PromiseEntity, union::UnionEntity},
};
use oxc::semantic::ScopeId;

#[derive(Debug)]
pub struct FunctionScope<'a> {
  /// `None` for indeterminate
  pub returned_values: Vec<Entity<'a>>,
  pub cf_scope_id: ScopeId,
  pub this: Entity<'a>,
  pub is_async: bool,
  pub has_await_effect: bool,
  pub try_scopes: Vec<TryScope<'a>>,
}

impl<'a> FunctionScope<'a> {
  pub fn new(cf_scope_id: ScopeId, this: Entity<'a>, is_async: bool) -> Self {
    FunctionScope {
      returned_values: Vec::new(),
      cf_scope_id,
      this,
      is_async,
      has_await_effect: false,
      try_scopes: vec![TryScope::new(cf_scope_id)],
    }
  }

  pub fn ret_val(self, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    assert_eq!(self.try_scopes.len(), 1);
    self.try_scopes.into_iter().next().unwrap().thrown_val().consume_as_unknown(analyzer);

    let value = if self.returned_values.is_empty() {
      LiteralEntity::new_undefined()
    } else {
      UnionEntity::new(self.returned_values)
    };
    if self.is_async {
      PromiseEntity::new(self.has_await_effect, value)
    } else {
      value
    }
  }
}
