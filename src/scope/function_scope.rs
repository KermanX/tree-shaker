use crate::entity::{entity::Entity, union::UnionEntity};
use oxc::ast::ast::{ArrowFunctionExpression, Function};

#[derive(Debug)]
pub(crate) enum FunctionScopeNode<'a> {
  Function(&'a Function<'a>),
  Arrow(&'a ArrowFunctionExpression<'a>),
}

#[derive(Debug)]
pub(crate) struct FunctionScope<'a> {
  /// `None` for indeterminate
  pub(crate) returned: Option<bool>,
  pub(crate) returned_value: Vec<Entity<'a>>,
}

impl<'a> FunctionScope<'a> {
  pub(crate) fn new() -> Self {
    FunctionScope { returned: None, returned_value: Vec::new() }
  }

  pub(crate) fn returned_value(self) -> Entity<'a> {
    UnionEntity::new(self.returned_value)
  }
}
