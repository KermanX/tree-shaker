use crate::EcmaAnalyzer;

use super::try_scope::TryScope;
use oxc::{
  ast::ast::{ArrowFunctionExpression, Class, Function},
  semantic::ScopeId,
  span::{GetSpan, Span},
};
use std::{hash, mem};

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
  pub debug_name: &'a str,
}

pub struct CallScope<'a, A: EcmaAnalyzer<'a> + ?Sized> {
  pub call_id: usize,
  pub callee: CalleeInfo<'a>,
  pub old_variable_scope_stack: Vec<ScopeId>,
  pub cf_scope_depth: usize,
  pub body_variable_scope: ScopeId,
  pub returned_values: Vec<A::Entity>,
  pub is_async: bool,
  pub is_generator: bool,
  pub try_scopes: Vec<TryScope<'a, A>>,
  pub need_consume_arguments: bool,

  #[cfg(feature = "flame")]
  pub scope_guard: flame::SpanGuard,
}

pub trait CallScopeAnalyzer<'a> {
  fn new_call_scope(
    call_id: usize,
    callee: CalleeInfo<'a>,
    old_variable_scope_stack: Vec<ScopeId>,
    cf_scope_depth: usize,
    body_variable_scope: ScopeId,
    is_async: bool,
    is_generator: bool,
  ) -> Self
  where
    Self: EcmaAnalyzer<'a>,
  {
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

      #[cfg(feature = "flame")]
      scope_guard: flame::start_guard(callee.debug_name.to_string()),
    }
  }

  fn finalize_call_scope(&mut self, scope: CallScope<'a, Self>) -> (Vec<ScopeId>, Self::Entity)
  where
    Self: EcmaAnalyzer<'a>,
  {
    assert_eq!(scope.try_scopes.len(), 1);

    // Forwards the thrown value to the parent try scope
    let try_scope = scope.try_scopes.into_iter().next().unwrap();
    let mut promise_error = None;
    if try_scope.may_throw {
      if scope.is_generator {
        let unknown = analyzer.factory.unknown();
        let parent_try_scope = analyzer.try_scope_mut();
        parent_try_scope.may_throw = true;
        if !try_scope.thrown_values.is_empty() {
          parent_try_scope.thrown_values.push(unknown);
        }
        for value in try_scope.thrown_values {
          value.consume(analyzer);
        }
      } else if scope.is_async {
        promise_error = Some(try_scope.thrown_values);
      } else {
        analyzer.forward_throw(try_scope.thrown_values);
      }
    }

    let value = if scope.returned_values.is_empty() {
      analyzer.factory.undefined
    } else {
      analyzer.factory.union(scope.returned_values)
    };

    let value = if scope.is_async {
      analyzer.factory.computed_unknown(analyzer.consumable((value, promise_error)))
    } else {
      value
    };

    #[cfg(feature = "flame")]
    scope.scope_guard.end();

    (scope.old_variable_scope_stack, value)
  }

  fn consume_arguments(&mut self) -> bool
  where
    Self: EcmaAnalyzer<'a>,
  {
    let scope = self.call_scope().body_variable_scope;
    self.consume_arguments_on_scope(scope)
  }

  fn consume_return_values(&mut self)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let call_scope = self.call_scope_mut();
    let values = mem::take(&mut call_scope.returned_values);
    for value in values {
      self.consume(value);
    }
  }
}
