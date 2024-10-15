use super::try_scope::TryScope;
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, ConsumableTrait},
  dep::DepId,
  entity::{Entity, FunctionEntitySource},
};
use oxc::semantic::{ScopeId, SymbolId};
use std::mem;

#[derive(Debug)]
pub struct CallScope<'a> {
  pub dep_id: DepId,
  pub source: FunctionEntitySource<'a>,
  pub old_variable_scope_stack: Vec<ScopeId>,
  pub cf_scope_depth: usize,
  pub body_variable_scope: ScopeId,
  pub this: Entity<'a>,
  pub args: (Entity<'a>, Vec<SymbolId>),
  pub returned_values: Vec<Entity<'a>>,
  pub is_async: bool,
  pub is_generator: bool,
  pub try_scopes: Vec<TryScope<'a>>,
  pub need_consume_arguments: bool,
}

impl<'a> CallScope<'a> {
  pub fn new(
    dep_id: DepId,
    source: FunctionEntitySource<'a>,
    old_variable_scope_stack: Vec<ScopeId>,
    cf_scope_depth: usize,
    body_variable_scope: ScopeId,
    this: Entity<'a>,
    args: (Entity<'a>, Vec<SymbolId>),
    is_async: bool,
    is_generator: bool,
  ) -> Self {
    CallScope {
      dep_id,
      source,
      old_variable_scope_stack,
      cf_scope_depth,
      body_variable_scope,
      this,
      args,
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
        let unknown = analyzer.factory.unknown;
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
      if self.is_async { analyzer.factory.promise(value, promise_error) } else { value },
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

  pub fn consume_arguments(&mut self, search: Option<FunctionEntitySource<'a>>) -> bool {
    let call_scope = if let Some(source) = search {
      if let Some(call_scope) =
        self.scope_context.call.iter().rev().find(|scope| scope.source == source)
      {
        call_scope
      } else {
        return false;
      }
    } else {
      self.call_scope()
    };
    let body_variable_scope = call_scope.body_variable_scope;
    let (args_entity, args_symbols) = call_scope.args.clone();
    args_entity.consume(self);
    let mut arguments_consumed = true;
    for symbol in args_symbols {
      if !self.consume_on_scope(body_variable_scope, symbol) {
        // Still inside parameter declaration
        arguments_consumed = false;
      }
    }
    arguments_consumed
  }

  pub fn consume_return_values(&mut self) {
    let call_scope = self.call_scope_mut();
    let values = mem::take(&mut call_scope.returned_values);
    for value in values {
      self.consume(value);
    }
  }
}
