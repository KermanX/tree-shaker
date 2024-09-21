use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  data::{get_node_ptr, ReferredNodes},
  transformer::Transformer,
};
use core::hash::Hash;
use oxc::{ast::AstKind, span::GetSpan};
use std::{fmt::Debug, rc::Rc};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityDepNode {
  Environment,
  AstKind((usize, usize)),
  Ptr(AstType2, usize),
}

impl Debug for EntityDepNode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EntityDepNode::Environment => {
        f.write_str("Environment")?;
      }
      EntityDepNode::AstKind(node) => {
        let node = unsafe { std::mem::transmute::<_, AstKind<'static>>(*node) };
        node.span().fmt(f)?;
      }
      EntityDepNode::Ptr(t, s) => {
        (*t).fmt(f)?;
        s.fmt(f)?;
      }
    }
    Ok(())
  }
}

impl<'a> From<AstKind<'a>> for EntityDepNode {
  fn from(node: AstKind<'a>) -> Self {
    EntityDepNode::AstKind(unsafe { std::mem::transmute(node) })
  }
}

impl<T: GetSpan> From<(AstType2, &T)> for EntityDepNode {
  fn from((ty, node): (AstType2, &T)) -> Self {
    EntityDepNode::Ptr(ty, get_node_ptr(node))
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
  pub fn mark_referred<'a>(&self, host: &mut ReferredNodes<'a>) {
    match self.0.as_ref() {
      EntityDepImpl::Environment => {}
      EntityDepImpl::Single(node) => {
        host.insert(*node);
      }
      EntityDepImpl::Multiple(nodes) => {
        for node in nodes.as_ref() {
          node.mark_referred(host);
        }
      }
      EntityDepImpl::Concat(node, dep) => {
        host.insert(*node);
        dep.mark_referred(host);
      }
      EntityDepImpl::Combined(dep1, dep2) => {
        dep1.mark_referred(host);
        dep2.mark_referred(host);
      }
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn refer_dep(&mut self, dep: impl Into<EntityDep>) {
    dep.into().mark_referred(&mut self.referred_nodes);
  }
}

impl<'a> Transformer<'a> {
  pub fn refer_dep(&self, dep: impl Into<EntityDep>) {
    let mut referred_nodes = self.referred_nodes.borrow_mut();
    dep.into().mark_referred(&mut referred_nodes);
  }

  pub fn is_referred(&self, dep: impl Into<EntityDepNode>) -> bool {
    self.referred_nodes.borrow().contains(&dep.into())
  }
}
