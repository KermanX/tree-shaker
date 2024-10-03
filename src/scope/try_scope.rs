use crate::{
  analyzer::Analyzer,
  entity::{Entity, ForwardedEntity, UnknownEntity},
};

#[derive(Debug)]
pub struct TryScope<'a> {
  pub may_throw: bool,
  pub thrown_values: Vec<Entity<'a>>,
  /// Here we use index in current stack instead of ScopeId
  pub cf_scope_depth: usize,
  /// Here we use index in current stack instead of ScopeId
  pub variable_scope_depth: usize,
}

impl<'a> TryScope<'a> {
  pub fn new(cf_scope_depth: usize, variable_scope_depth: usize) -> Self {
    TryScope { may_throw: false, thrown_values: Vec::new(), cf_scope_depth, variable_scope_depth }
  }

  pub fn thrown_val(self) -> Option<Entity<'a>> {
    // Always unknown here
    self.may_throw.then(|| UnknownEntity::new_computed_unknown(self.thrown_values))
  }
}

impl<'a> Analyzer<'a> {
  pub fn may_throw(&mut self) {
    let try_scope = self.try_scope_mut();

    try_scope.may_throw = true;

    // FIXME: Some of the tests are failing because of this
    // let cf_scope_depth = try_scope.cf_scope_depth;
    // self.exit_to(cf_scope_depth, false);
  }

  pub fn explicit_throw(&mut self, value: Entity<'a>) {
    let try_scope = self.try_scope();
    let value =
      ForwardedEntity::new(value, self.get_assignment_deps(try_scope.variable_scope_depth, ()));
    self.explicit_throw_impl(value);
  }

  pub fn explicit_throw_unknown(&mut self, message: impl Into<String>) {
    if self.scope_context.cf.iter_stack().rev().all(|scope| scope.exited == Some(false)) {
      self.add_diagnostic(message);
    }

    let try_scope = self.try_scope();
    let value = UnknownEntity::new_computed_unknown(
      self.get_assignment_deps(try_scope.variable_scope_depth, ()),
    );
    self.explicit_throw_impl(value);
  }

  pub fn forward_throw(&mut self, values: Vec<Entity<'a>>) {
    if values.is_empty() {
      self.may_throw();
    } else {
      let thrown_val = UnknownEntity::new_computed_unknown(values);
      self.explicit_throw_impl(thrown_val);
    }
  }

  fn explicit_throw_impl(&mut self, value: Entity<'a>) {
    let try_scope = self.try_scope_mut();

    try_scope.may_throw = true;
    try_scope.thrown_values.push(value);

    let cf_scope_depth = try_scope.cf_scope_depth;
    self.exit_to(cf_scope_depth);
  }
}
