use crate::{analyzer::Analyzer, ast::AstType2, data::get_node_ptr, transformer::Transformer};
use oxc::{ast::AstKind, span::GetSpan};
use std::{
  fmt::Debug,
  hash::Hash,
  sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepId {
  Environment,
  AstKind((usize, usize)),
  AstType(AstType2, usize),
  Index(usize),
}

impl Debug for DepId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DepId::Environment => {
        f.write_str("Environment")?;
      }
      DepId::AstKind(node) => {
        let node = unsafe { std::mem::transmute::<_, AstKind<'static>>(*node) };
        node.span().fmt(f)?;
      }
      DepId::AstType(t, s) => {
        (*t).fmt(f)?;
        s.fmt(f)?;
      }
      DepId::Index(c) => {
        c.fmt(f)?;
      }
    }
    Ok(())
  }
}

impl<'a> From<AstKind<'a>> for DepId {
  fn from(node: AstKind<'a>) -> Self {
    DepId::AstKind(unsafe { std::mem::transmute(node) })
  }
}

impl<T: GetSpan> From<(AstType2, &T)> for DepId {
  fn from((ty, node): (AstType2, &T)) -> Self {
    DepId::AstType(ty, get_node_ptr(node))
  }
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

impl DepId {
  pub fn from_counter() -> Self {
    DepId::Index(COUNTER.fetch_add(1, Ordering::Relaxed))
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
