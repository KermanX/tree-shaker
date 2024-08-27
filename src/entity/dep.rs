use core::hash::{Hash, Hasher};
use oxc::{
  ast::ast::{
    ArrowFunctionExpression, BindingIdentifier, Function, LabelIdentifier, ReturnStatement,
    SimpleAssignmentTarget,
  },
  semantic::ScopeId,
  span::GetSpan,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum EntityDepNode<'a> {
  Function(&'a Function<'a>),
  ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>),
  BindingIdentifier(&'a BindingIdentifier<'a>),
  ReturnStatement(&'a ReturnStatement<'a>),
  LabelIdentifier(&'a LabelIdentifier<'a>),
  SimpleAssignmentTarget(&'a SimpleAssignmentTarget<'a>),
}

#[derive(Debug, Clone)]
pub(crate) struct EntityDep<'a> {
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
    };
    span.hash(state);
  }
}
