use crate::{analyzer::Analyzer, ast::AstKind2, transformer::Transformer};
use oxc::span::{GetSpan, Span};
use std::{
  fmt::Debug,
  hash::Hash,
  sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepId((usize, usize));

impl Debug for DepId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.span().fmt(f)
  }
}

impl<'a> From<AstKind2<'a>> for DepId {
  fn from(node: AstKind2<'a>) -> Self {
    DepId(unsafe { std::mem::transmute(node) })
  }
}

impl<'a> GetSpan for DepId {
  fn span(&self) -> Span {
    let ast_kind: AstKind2<'static> = unsafe { std::mem::transmute(*self) };
    ast_kind.span()
  }
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

impl DepId {
  pub fn from_counter() -> Self {
    AstKind2::Index(COUNTER.fetch_add(1, Ordering::Relaxed)).into()
  }
}

impl<'a> Analyzer<'a> {
  pub fn refer_dep(&mut self, dep: impl Into<DepId>) {
    self.referred_nodes.entry(dep.into()).and_modify(|v| *v += 1).or_insert(1);
  }

  pub fn is_referred(&self, dep: impl Into<DepId>) -> bool {
    self.referred_nodes.contains_key(&dep.into())
  }
}

impl<'a> Transformer<'a> {
  pub fn is_referred(&self, dep: impl Into<DepId>) -> bool {
    self.referred_nodes.contains_key(&dep.into())
  }
}
