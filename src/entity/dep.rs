use crate::ast::AstType2;
use core::hash::Hash;
use oxc::{
  ast::AstKind,
  span::{GetSpan, Span},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityDep {
  Environment,
  AstKind(u128),
  Span(AstType2, Span),
}

impl<'a> From<AstKind<'a>> for EntityDep {
  fn from(node: AstKind<'a>) -> Self {
    EntityDep::AstKind(unsafe { std::mem::transmute(node) })
  }
}

impl<T: GetSpan> From<(AstType2, &T)> for EntityDep {
  fn from((ty, node): (AstType2, &T)) -> Self {
    EntityDep::Span(ty, node.span())
  }
}
