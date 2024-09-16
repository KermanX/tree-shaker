use crate::{analyzer::Analyzer, ast::AstType2, data::get_node_ptr};
use core::hash::Hash;
use oxc::{ast::AstKind, span::GetSpan};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityDepNode {
  Environment,
  AstKind(u128),
  Span(AstType2, usize),
}

impl<'a> From<AstKind<'a>> for EntityDepNode {
  fn from(node: AstKind<'a>) -> Self {
    EntityDepNode::AstKind(unsafe { std::mem::transmute(node) })
  }
}

impl<T: GetSpan> From<(AstType2, &T)> for EntityDepNode {
  fn from((ty, node): (AstType2, &T)) -> Self {
    EntityDepNode::Span(ty, get_node_ptr(node))
  }
}

#[derive(Debug, Clone)]
enum EntityDepImpl {
  Environment,
  Single(EntityDepNode),
  Multiple(Rc<Vec<EntityDep>>),
  Concat(EntityDepNode, EntityDep),
  Combined(EntityDep, EntityDep),
}

#[derive(Debug, Clone)]
pub struct EntityDep(Rc<EntityDepImpl>);

impl From<()> for EntityDep {
  fn from(_: ()) -> Self {
    Self(Rc::new(EntityDepImpl::Environment))
  }
}

impl From<EntityDepNode> for EntityDep {
  fn from(node: EntityDepNode) -> Self {
    Self(Rc::new(EntityDepImpl::Single(node)))
  }
}

impl From<Vec<EntityDep>> for EntityDep {
  fn from(deps: Vec<EntityDep>) -> Self {
    Self(Rc::new(EntityDepImpl::Multiple(Rc::new(deps))))
  }
}

impl From<(EntityDepNode, EntityDep)> for EntityDep {
  fn from((node, dep): (EntityDepNode, EntityDep)) -> Self {
    Self(Rc::new(EntityDepImpl::Concat(node, dep)))
  }
}

impl From<(EntityDep, EntityDep)> for EntityDep {
  fn from((dep1, dep2): (EntityDep, EntityDep)) -> Self {
    Self(Rc::new(EntityDepImpl::Combined(dep1, dep2)))
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
      EntityDepImpl::Environment => {}
      EntityDepImpl::Single(node) => {
        analyzer.referred_nodes.insert(*node);
      }
      EntityDepImpl::Multiple(nodes) => {
        for node in nodes.as_ref() {
          node.mark_referred(analyzer);
        }
      }
      EntityDepImpl::Concat(node, dep) => {
        analyzer.referred_nodes.insert(*node);
        dep.mark_referred(analyzer);
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
