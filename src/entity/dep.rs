use core::hash::{Hash, Hasher};
use oxc::{
  ast::ast::{
    Argument, ArrowFunctionExpression, BindingIdentifier, Function, FunctionBody, LabelIdentifier,
    ReturnStatement, SimpleAssignmentTarget, ThrowStatement,
  },
  semantic::ScopeId,
  span::GetSpan,
};

#[derive(Debug, Clone, Copy)]
pub enum EntityDepNode<'a> {
  Function(&'a Function<'a>),
  ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>),
  BindingIdentifier(&'a BindingIdentifier<'a>),
  ReturnStatement(&'a ReturnStatement<'a>),
  LabelIdentifier(&'a LabelIdentifier<'a>),
  SimpleAssignmentTarget(&'a SimpleAssignmentTarget<'a>),
  FunctionBodyAsExpression(&'a FunctionBody<'a>),
  Argument(&'a Argument<'a>),
  ThrowStatement(&'a ThrowStatement<'a>),
}

#[derive(Debug, Clone)]
pub struct EntityDep<'a> {
  pub node: EntityDepNode<'a>,
  pub scope_path: Vec<ScopeId>,
}

impl<'a> PartialEq for EntityDepNode<'a> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (EntityDepNode::Function(a), EntityDepNode::Function(b)) => a.span() == b.span(),
      (EntityDepNode::ArrowFunctionExpression(a), EntityDepNode::ArrowFunctionExpression(b)) => {
        a.span() == b.span()
      }
      (EntityDepNode::BindingIdentifier(a), EntityDepNode::BindingIdentifier(b)) => {
        a.span() == b.span()
      }
      (EntityDepNode::ReturnStatement(a), EntityDepNode::ReturnStatement(b)) => {
        a.span() == b.span()
      }
      (EntityDepNode::LabelIdentifier(a), EntityDepNode::LabelIdentifier(b)) => {
        a.span() == b.span()
      }
      (EntityDepNode::SimpleAssignmentTarget(a), EntityDepNode::SimpleAssignmentTarget(b)) => {
        a.span() == b.span()
      }
      (EntityDepNode::FunctionBodyAsExpression(a), EntityDepNode::FunctionBodyAsExpression(b)) => {
        a.span() == b.span()
      }
      (EntityDepNode::Argument(a), EntityDepNode::Argument(b)) => a.span() == b.span(),
      (EntityDepNode::ThrowStatement(a), EntityDepNode::ThrowStatement(b)) => a.span() == b.span(),
      _ => false,
    }
  }
}

impl<'a> Eq for EntityDepNode<'a> {}

impl<'a> Hash for EntityDepNode<'a> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    let span = match self {
      EntityDepNode::Function(a) => a.span(),
      EntityDepNode::ArrowFunctionExpression(a) => a.span(),
      EntityDepNode::BindingIdentifier(a) => a.span(),
      EntityDepNode::ReturnStatement(a) => a.span(),
      EntityDepNode::LabelIdentifier(a) => a.span(),
      EntityDepNode::SimpleAssignmentTarget(a) => a.span(),
      EntityDepNode::FunctionBodyAsExpression(a) => a.span(),
      EntityDepNode::Argument(a) => a.span(),
      EntityDepNode::ThrowStatement(a) => a.span(),
    };
    span.hash(state);
  }
}
