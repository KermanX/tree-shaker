use crate::{analyzer::Analyzer, ast::AstType2};
use core::hash::Hash;
use oxc::{
  ast::AstKind,
  span::{GetSpan, Span},
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityDepNode {
  Environment,
  AstKind(u128),
  Span(AstType2, Span),
}

impl<'a> From<AstKind<'a>> for EntityDepNode {
  fn from(node: AstKind<'a>) -> Self {
    EntityDepNode::AstKind(unsafe { std::mem::transmute(node) })
  }
}

impl<T: GetSpan> From<(AstType2, &T)> for EntityDepNode {
  fn from((ty, node): (AstType2, &T)) -> Self {
    EntityDepNode::Span(ty, node.span())
  }
}

#[derive(Debug, Clone)]
pub enum EntityDepImpl {
  Single(EntityDepNode),
  Multiple(Vec<EntityDepNode>),
  Combined(EntityDep, EntityDep),
}

#[derive(Debug, Clone)]
pub struct EntityDep(Rc<EntityDepImpl>);

impl From<EntityDepNode> for EntityDep {
  fn from(node: EntityDepNode) -> Self {
    Self(Rc::new(EntityDepImpl::Single(node)))
  }
}

impl<'a> From<AstKind<'a>> for EntityDep {
  fn from(node: AstKind<'a>) -> Self {
    EntityDepNode::from(node).into()
  }
}

impl EntityDep {
  pub fn mark_referred(&self, analyzer: &mut Analyzer) {
    match self.0.as_ref() {
      EntityDepImpl::Single(node) => {
        analyzer.referred_nodes.insert(*node);
      }
      EntityDepImpl::Multiple(nodes) => {
        for node in nodes {
          analyzer.referred_nodes.insert(*node);
        }
      }
      EntityDepImpl::Combined(dep1, dep2) => {
        dep1.mark_referred(analyzer);
        dep2.mark_referred(analyzer);
      }
    }
  }
}

impl Analyzer<'_> {
  pub fn refer_dep(&mut self, dep: impl Into<EntityDep>) {
    dep.into().mark_referred(self);
  }
}
