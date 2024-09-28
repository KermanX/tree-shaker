use super::{try_scope::TryScope, variable_scope::VariableScopes};
use crate::{
  analyzer::Analyzer,
  entity::{
    Consumable, Entity, EntityDepNode, ForwardedEntity, LiteralEntity, PromiseEntity, UnionEntity,
    UnknownEntity,
  },
};
use oxc::semantic::SymbolId;

#[derive(Debug)]
pub struct CallScope<'a> {
  pub source: EntityDepNode,
  pub call_dep: Consumable<'a>,
  pub old_variable_scopes: VariableScopes<'a>,
  pub cf_scope_index: usize,
  pub variable_scope_index: usize,
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
    source: EntityDepNode,
    call_dep: Consumable<'a>,
    old_variable_scopes: VariableScopes<'a>,
    cf_scope_index: usize,
    variable_scope_index: usize,
    this: Entity<'a>,
    args: (Entity<'a>, Vec<SymbolId>),
    is_async: bool,
    is_generator: bool,
  ) -> Self {
    CallScope {
      source,
      call_dep,
      old_variable_scopes,
      cf_scope_index,
      variable_scope_index,
      this,
      args,
      returned_values: Vec::new(),
      is_async,
      is_generator,
      try_scopes: vec![TryScope::new(cf_scope_index, variable_scope_index)],
      need_consume_arguments: false,
    }
  }

  pub fn finalize(self, analyzer: &mut Analyzer<'a>) -> (VariableScopes<'a>, Entity<'a>) {
    assert_eq!(self.try_scopes.len(), 1);

    // Forwards the thrown value to the parent try scope
    let try_scope = self.try_scopes.into_iter().next().unwrap();
    let mut promise_error = None;
    if try_scope.may_throw {
      if self.is_generator {
        let parent_try_scope = analyzer.try_scope_mut();
        parent_try_scope.may_throw = true;
        if !try_scope.thrown_values.is_empty() {
          parent_try_scope
            .thrown_values
            .push(ForwardedEntity::new(UnknownEntity::new_unknown(), self.call_dep.clone()));
        }
        for value in try_scope.thrown_values {
          value.consume(analyzer);
        }
      } else if self.is_async {
        promise_error = Some(try_scope.thrown_values);
      } else {
        analyzer.forward_throw(try_scope.thrown_values, self.call_dep.clone());
      }
    }

    let value = if self.returned_values.is_empty() {
      LiteralEntity::new_undefined()
    } else {
      UnionEntity::new(self.returned_values)
    };
    (
      self.old_variable_scopes,
      if self.is_async { PromiseEntity::new(value, promise_error, self.call_dep) } else { value },
    )
  }
}

impl<'a> Analyzer<'a> {
  pub fn consume_arguments(&mut self) -> bool {
    let mut arguments_consumed = true;
    let (args_entity, args_symbols) = self.call_scope().args.clone();
    args_entity.consume(self);
    for symbol in args_symbols {
      // FIXME: Accessing `arguments` in formal parameters
      if let Some((_, variable_scopes, decl_dep)) = self.symbol_decls.get(&symbol) {
        let decl_dep = decl_dep.clone();
        let variable_scope = variable_scopes.last().unwrap().clone();
        variable_scope.consume(self, symbol);
        self.consume(decl_dep);
      } else {
        // TDZ
        arguments_consumed = false;
      }
    }
    arguments_consumed
  }
}
