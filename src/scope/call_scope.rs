use super::try_scope::TryScope;
use crate::{
  analyzer::Analyzer,
  ast::AstKind2,
  consumable::{box_consumable, ConsumableNode, ConsumableTrait},
  dep::DepId,
  entity::Entity,
};
use oxc::{
  ast::ast::{ArrowFunctionExpression, Class, Function},
  semantic::ScopeId,
  span::{GetSpan, Span},
};
use std::{hash, mem};

#[derive(Debug)]
pub struct CallScope<'a> {
  pub call_id: DepId,
  pub callee: (CalleeNode<'a>, usize),
  pub old_variable_scope_stack: Vec<ScopeId>,
  pub cf_scope_depth: usize,
  pub body_variable_scope: ScopeId,
  pub returned_values: Vec<Entity<'a>>,
  pub is_async: bool,
  pub is_generator: bool,
  pub try_scopes: Vec<TryScope<'a>>,
  pub need_consume_arguments: bool,
}

impl<'a> CallScope<'a> {
  pub fn new(
    call_id: DepId,
    callee: (CalleeNode<'a>, usize),
    old_variable_scope_stack: Vec<ScopeId>,
    cf_scope_depth: usize,
    body_variable_scope: ScopeId,
    is_async: bool,
    is_generator: bool,
  ) -> Self {
    CallScope {
      call_id,
      callee,
      old_variable_scope_stack,
      cf_scope_depth,
      body_variable_scope,
      returned_values: Vec::new(),
      is_async,
      is_generator,
      try_scopes: vec![TryScope::new(cf_scope_depth)],
      need_consume_arguments: false,
    }
  }

  pub fn finalize(self, analyzer: &mut Analyzer<'a>) -> (Vec<ScopeId>, Entity<'a>) {
    assert_eq!(self.try_scopes.len(), 1);

    // Forwards the thrown value to the parent try scope
    let try_scope = self.try_scopes.into_iter().next().unwrap();
    let mut promise_error = None;
    if try_scope.may_throw {
      if self.is_generator {
        let unknown = analyzer.factory.unknown();
        let parent_try_scope = analyzer.try_scope_mut();
        parent_try_scope.may_throw = true;
        if !try_scope.thrown_values.is_empty() {
          parent_try_scope.thrown_values.push(unknown);
        }
        for value in try_scope.thrown_values {
          value.consume(analyzer);
        }
      } else if self.is_async {
        promise_error = Some(try_scope.thrown_values);
      } else {
        analyzer.forward_throw(try_scope.thrown_values);
      }
    }

    let value = if self.returned_values.is_empty() {
      analyzer.factory.undefined
    } else {
      analyzer.factory.union(self.returned_values)
    };
    (
      self.old_variable_scope_stack,
      if self.is_async {
        analyzer.factory.computed_unknown(ConsumableNode::new((value, promise_error)))
      } else {
        value
      },
    )
  }
}

impl<'a> Analyzer<'a> {
  pub fn return_value(&mut self, value: Entity<'a>, dep: impl ConsumableTrait<'a> + 'a) {
    let call_scope = self.call_scope();
    let dep = box_consumable((self.get_exec_dep(call_scope.cf_scope_depth), dep));
    let value = self.factory.computed(value, dep);

    let call_scope = self.call_scope_mut();
    call_scope.returned_values.push(value);

    let target_depth = call_scope.cf_scope_depth;
    self.exit_to(target_depth);
  }

  pub fn consume_arguments(&mut self, search: Option<(CalleeNode<'a>, usize)>) -> bool {
    let call_scope = if let Some(callee) = search {
      if let Some(call_scope) =
        self.scope_context.call.iter().rev().find(|scope| scope.callee.1 == callee.1)
      {
        call_scope
      } else {
        return false;
      }
    } else {
      self.call_scope()
    };
    self.consume_arguments_on_scope(call_scope.body_variable_scope)
  }

  pub fn consume_return_values(&mut self) {
    let call_scope = self.call_scope_mut();
    let values = mem::take(&mut call_scope.returned_values);
    for value in values {
      self.consume(value);
    }
  }
}

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

impl<'a> CalleeNode<'a> {
  pub fn into_dep_id(self) -> DepId {
    match self {
      CalleeNode::Function(node) => AstKind2::Function(node),
      CalleeNode::ArrowFunctionExpression(node) => AstKind2::ArrowFunctionExpression(node),
      CalleeNode::ClassStatics(node) => AstKind2::Class(node),
      CalleeNode::ClassConstructor(node) => AstKind2::Class(node),
      CalleeNode::Module => AstKind2::Environment,
    }
    .into()
  }

  pub fn name(&self) -> String {
    match self {
      CalleeNode::Function(node) => node.id.as_ref().map_or("<unknown>", |id| &id.name).to_string(),
      CalleeNode::ArrowFunctionExpression(_) => "<anonymous>".to_string(),
      CalleeNode::ClassStatics(_) => "<ClassStatics>".to_string(),
      CalleeNode::ClassConstructor(_) => "<ClassConstructor>".to_string(),
      CalleeNode::Module => "<Module>".to_string(),
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
