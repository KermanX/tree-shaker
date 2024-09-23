use super::{try_scope::TryScope, variable_scope::VariableScopes};
use crate::{
  analyzer::Analyzer,
  entity::{
    dep::{EntityDep, EntityDepNode},
    entity::Entity,
    forwarded::ForwardedEntity,
    literal::LiteralEntity,
    promise::PromiseEntity,
    union::UnionEntity,
    unknown::UnknownEntity,
  },
};
use oxc::semantic::SymbolId;

#[derive(Debug)]
pub struct CallScope<'a> {
  pub source: EntityDepNode,
  pub call_dep: EntityDep,
  pub old_variable_scopes: VariableScopes<'a>,
  pub cf_scope_index: usize,
  pub variable_scope_index: usize,
  pub this: Entity<'a>,
  pub args: (Entity<'a>, Vec<SymbolId>),
  pub returned_values: Vec<Entity<'a>>,
  pub is_async: bool,
  pub await_has_effect: bool,
  pub try_scopes: Vec<TryScope<'a>>,
  pub is_generator: bool,
  pub need_consume_arguments: bool,
}

impl<'a> CallScope<'a> {
  pub fn new(
    source: EntityDepNode,
    call_dep: EntityDep,
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
      await_has_effect: false,
      try_scopes: vec![TryScope::new(cf_scope_index)],
      is_generator,
      need_consume_arguments: false,
    }
  }

  pub fn finalize(self, analyzer: &mut Analyzer<'a>) -> (VariableScopes<'a>, Entity<'a>) {
    assert_eq!(self.try_scopes.len(), 1);

    // Forwards the thrown value to the parent try scope
    let try_scope = self.try_scopes.into_iter().next().unwrap();
    let promise_error = try_scope.thrown_val().and_then(|thrown_val| {
      let thrown_val = ForwardedEntity::new(thrown_val, self.call_dep);
      if self.is_async {
        Some(thrown_val)
      } else {
        analyzer.try_scope_mut().throw(thrown_val);
        None
      }
    });

    if self.is_generator {
      for value in &self.returned_values {
        value.consume(analyzer);
      }
      return (self.old_variable_scopes, UnknownEntity::new_unknown());
    }

    let value = if self.returned_values.is_empty() {
      LiteralEntity::new_undefined()
    } else {
      UnionEntity::new(self.returned_values)
    };
    (
      self.old_variable_scopes,
      if self.is_async {
        PromiseEntity::new(self.await_has_effect, value, promise_error)
      } else {
        value
      },
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
      if let Some(old) = self.read_symbol(&symbol) {
        self.write_symbol(&symbol, UnknownEntity::new_unknown_with_deps(vec![old]));
      } else {
        // TDZ
        arguments_consumed = false
      }
    }
    arguments_consumed
  }
}
