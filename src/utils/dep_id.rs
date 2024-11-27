use crate::{
  analyzer::Analyzer,
  ast::AstKind2,
  consumable::{Consumable, ConsumableTrait},
  transformer::Transformer,
};
use oxc::span::{GetSpan, Span};
use rustc_hash::FxHashMap;
use std::{
  fmt::{self, Debug},
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

impl<'a> Into<AstKind2<'a>> for DepId {
  fn into(self) -> AstKind2<'a> {
    unsafe { std::mem::transmute(self.0) }
  }
}

impl<'a> GetSpan for DepId {
  fn span(&self) -> Span {
    let ast_kind: AstKind2<'a> = (*self).into();
    ast_kind.span()
  }
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

impl DepId {
  pub fn from_counter() -> Self {
    AstKind2::Index(COUNTER.fetch_add(1, Ordering::Relaxed)).into()
  }
}

#[derive(Default)]
pub struct ReferredDeps {
  by_index: Vec<usize>,
  by_ptr: FxHashMap<DepId, usize>,
}

impl ReferredDeps {
  pub fn refer_dep(&mut self, dep: impl Into<DepId>) {
    let dep = dep.into();
    match dep.into() {
      AstKind2::Environment => {}
      AstKind2::Index(index) => {
        let counter = COUNTER.load(Ordering::Relaxed);
        if counter >= self.by_index.len() {
          self.by_index.resize(2 * counter, 0);
        };
        self.by_index[index] += 1;
      }
      _ => {
        *self.by_ptr.entry(dep).or_insert(0) += 1;
      }
    }
  }

  pub fn is_referred(&self, dep: impl Into<DepId>) -> bool {
    let dep = dep.into();
    match dep.into() {
      AstKind2::Environment => unreachable!(),
      AstKind2::Index(index) => self.by_index.get(index).copied().is_some_and(|x| x > 0),
      _ => self.by_ptr.contains_key(&dep),
    }
  }

  pub fn debug_count(&self) -> usize {
    self.by_ptr.len() + self.by_index.iter().filter(|&&x| x > 0).count()
  }
}

impl<'a> Analyzer<'a> {
  pub fn refer_dep(&mut self, dep: impl Into<DepId>) {
    self.referred_deps.refer_dep(dep);
  }

  pub fn is_referred(&self, dep: impl Into<DepId>) -> bool {
    self.referred_deps.is_referred(dep)
  }
}

impl<'a> Transformer<'a> {
  pub fn is_referred(&self, dep: impl Into<DepId>) -> bool {
    self.referred_deps.is_referred(dep)
  }
}

impl<'a> fmt::Debug for ReferredDeps {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    "ReferencedDeps".fmt(f)
  }
}

impl<'a> ConsumableTrait<'a> for ReferredDeps {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    if self.by_index.len() > analyzer.referred_deps.by_index.len() {
      analyzer.referred_deps.by_index.resize(self.by_index.len(), 0);
    }
    for (i, v) in self.by_index.iter().enumerate() {
      analyzer.referred_deps.by_index[i] += v;
    }
    analyzer.referred_deps.by_ptr.extend(self.by_ptr.iter().map(|(k, v)| (k.clone(), *v)));
  }

  fn cloned(&self) -> Consumable<'a> {
    unreachable!("Should not clone ReferredDeps")
  }
}
