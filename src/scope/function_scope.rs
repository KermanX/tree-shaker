use super::try_scope::TryScope;
use crate::{
  analyzer::Analyzer,
  entity::{
    entity::Entity, literal::LiteralEntity, promise::PromiseEntity, union::UnionEntity,
    unknown::UnknownEntity,
  },
};

#[derive(Debug)]
pub struct FunctionScope<'a> {
  pub cf_scope_index: usize,
  pub variable_scope_index: usize,
  pub this: Entity<'a>,
  /// `None` for indeterminate
  pub returned_values: Vec<Entity<'a>>,
  pub is_async: bool,
  pub has_await_effect: bool,
  pub try_scopes: Vec<TryScope<'a>>,
  pub is_generator: bool,
}

impl<'a> FunctionScope<'a> {
  pub fn new(
    cf_scope_index: usize,
    variable_scope_index: usize,
    this: Entity<'a>,
    is_async: bool,
    is_generator: bool,
  ) -> Self {
    FunctionScope {
      cf_scope_index,
      variable_scope_index,
      this,
      returned_values: Vec::new(),
      is_async,
      has_await_effect: false,
      try_scopes: vec![TryScope::new(cf_scope_index)],
      is_generator,
    }
  }

  pub fn finalize(self, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    assert_eq!(self.try_scopes.len(), 1);

    // Does not track values thrown out of function scope
    let try_scope = self.try_scopes.into_iter().next().unwrap();
    let may_throw = try_scope.may_throw;
    try_scope.thrown_val().consume_as_unknown(analyzer);

    if may_throw {
      analyzer.may_throw();
    }

    if self.is_generator {
      for value in &self.returned_values {
        value.consume_as_unknown(analyzer);
      }
      return (true, UnknownEntity::new_unknown());
    }

    let value = if self.returned_values.is_empty() {
      LiteralEntity::new_undefined()
    } else {
      UnionEntity::new(self.returned_values)
    };
    (
      may_throw,
      if self.is_async { PromiseEntity::new(self.has_await_effect, value) } else { value },
    )
  }
}
