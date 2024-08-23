use crate::entity::EntityValue;
use oxc::ast::ast::{ArrowFunctionExpression, Function};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) enum FunctionScopeNode<'a> {
  Function(&'a Function<'a>),
  Arrow(&'a ArrowFunctionExpression<'a>),
}

#[derive(Debug)]
pub(crate) struct FunctionScope<'a> {
  pub(crate) node: FunctionScopeNode<'a>,
  /// `None` for indeterminate
  pub(crate) returned: Option<bool>,
  pub(crate) returned_value: Vec<Rc<EntityValue>>,
}

impl<'a> FunctionScope<'a> {
  pub(crate) fn new(node: FunctionScopeNode<'a>) -> Self {
    FunctionScope { node, returned: None, returned_value: Vec::new() }
  }
}
