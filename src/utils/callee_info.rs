use super::ast::AstKind2;
use crate::{analyzer::Analyzer, dep::DepId, entity::EntityFactory};
use oxc::{
  ast::ast::{ArrowFunctionExpression, Class, Function},
  span::{GetSpan, Span},
};
use std::hash;

#[derive(Debug, Clone, Copy)]
pub enum CalleeNode<'a> {
  Function(&'a Function<'a>),
  ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>),
  ClassStatics(&'a Class<'a>),
  ClassConstructor(&'a Class<'a>),
  Module,
}

impl GetSpan for CalleeNode<'_> {
  fn span(&self) -> Span {
    match self {
      CalleeNode::Function(node) => node.span(),
      CalleeNode::ArrowFunctionExpression(node) => node.span(),
      CalleeNode::ClassStatics(node) => node.span(),
      CalleeNode::ClassConstructor(node) => node.span(),
      CalleeNode::Module => Span::default(),
    }
  }
}

impl PartialEq for CalleeNode<'_> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (CalleeNode::Module, CalleeNode::Module) => true,
      (CalleeNode::Function(a), CalleeNode::Function(b)) => a.span() == b.span(),
      (CalleeNode::ArrowFunctionExpression(a), CalleeNode::ArrowFunctionExpression(b)) => {
        a.span() == b.span()
      }
      (CalleeNode::ClassStatics(a), CalleeNode::ClassStatics(b)) => a.span() == b.span(),
      _ => false,
    }
  }
}

impl Eq for CalleeNode<'_> {}

impl hash::Hash for CalleeNode<'_> {
  fn hash<H: hash::Hasher>(&self, state: &mut H) {
    self.span().hash(state)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct CalleeInfo<'a> {
  pub node: CalleeNode<'a>,
  pub instance_id: usize,
  #[cfg(feature = "flame")]
  pub line_col: line_index::LineCol,
}

impl<'a> CalleeInfo<'a> {
  pub fn new_module(factory: &EntityFactory<'a>) -> Self {
    Self {
      node: CalleeNode::Module,
      instance_id: factory.alloc_instance_id(),
      #[cfg(feature = "flame")]
      line_col: line_index::LineCol { line: 0, col: 0 },
    }
  }

  pub fn into_dep_id(self) -> DepId {
    match self.node {
      CalleeNode::Function(node) => AstKind2::Function(node),
      CalleeNode::ArrowFunctionExpression(node) => AstKind2::ArrowFunctionExpression(node),
      CalleeNode::ClassStatics(node) => AstKind2::Class(node),
      CalleeNode::ClassConstructor(node) => AstKind2::Class(node),
      CalleeNode::Module => AstKind2::Environment,
    }
    .into()
  }

  pub fn span(&self) -> Span {
    self.node.span()
  }

  pub fn name(&self) -> String {
    let name = match self.node {
      CalleeNode::Function(node) => node.id.as_ref().map_or("<unknown>", |id| &id.name).to_string(),
      CalleeNode::ArrowFunctionExpression(_) => "<anonymous>".to_string(),
      CalleeNode::ClassStatics(_) => "<ClassStatics>".to_string(),
      CalleeNode::ClassConstructor(_) => "<ClassConstructor>".to_string(),
      CalleeNode::Module => return "<Module>".to_string(),
    };
    #[cfg(feature = "flame")]
    {
      format!("{}:{}:{}", name, self.line_col.line, self.line_col.col)
    }
    #[cfg(not(feature = "flame"))]
    {
      name
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_callee_info(&self, node: CalleeNode<'a>) -> CalleeInfo<'a> {
    CalleeInfo {
      node,
      instance_id: self.factory.alloc_instance_id(),
      #[cfg(feature = "flame")]
      line_col: self.line_index.line_col(node.span().start.into()),
    }
  }
}
