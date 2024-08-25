use core::hash::{Hash, Hasher};
use oxc::{
  ast::ast::{ArrowFunctionExpression, BindingIdentifier, Function, ReturnStatement},
  span::GetSpan,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum EntityDep<'a> {
  Function(&'a Function<'a>),
  ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>),
  BindingIdentifier(&'a BindingIdentifier<'a>),
  ReturnStatement(&'a ReturnStatement<'a>),
}

impl<'a> PartialEq for EntityDep<'a> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (EntityDep::Function(a), EntityDep::Function(b)) => a.span() == b.span(),
      (EntityDep::ArrowFunctionExpression(a), EntityDep::ArrowFunctionExpression(b)) => {
        a.span() == b.span()
      }
      (EntityDep::BindingIdentifier(a), EntityDep::BindingIdentifier(b)) => a.span() == b.span(),
      (EntityDep::ReturnStatement(a), EntityDep::ReturnStatement(b)) => a.span() == b.span(),
      _ => false,
    }
  }
}

impl<'a> Eq for EntityDep<'a> {}

impl<'a> Hash for EntityDep<'a> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    let span = match self {
      EntityDep::Function(a) => a.span(),
      EntityDep::ArrowFunctionExpression(a) => a.span(),
      EntityDep::BindingIdentifier(a) => a.span(),
      EntityDep::ReturnStatement(a) => a.span(),
    };
    span.hash(state);
  }
}
