use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, ConsumableNode},
  entity::{Entity, ForwardedEntity, UnknownEntity},
};

#[derive(Debug)]
pub struct TryScope<'a> {
  pub may_throw: bool,
  pub thrown_values: Vec<Entity<'a>>,
  /// Here we use index in current stack instead of ScopeId
  pub cf_scope_depth: usize,
}

impl<'a> TryScope<'a> {
  pub fn new(cf_scope_depth: usize) -> Self {
    TryScope { may_throw: false, thrown_values: Vec::new(), cf_scope_depth }
  }

  pub fn thrown_val(self) -> Option<Entity<'a>> {
    // Always unknown here
    self.may_throw.then(|| UnknownEntity::new_computed_unknown(box_consumable(self.thrown_values)))
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
    self.explicit_throw_impl(value);

    let try_scope = self.try_scope();
    self.exit_to(try_scope.cf_scope_depth);
  }

  pub fn thrown_builtin_error(&mut self, message: impl Into<String>) {
    if self.scope_context.cf.iter_stack().all(|scope| scope.exited == Some(false)) {
      self.add_diagnostic(message);
    }

    self.explicit_throw_impl(UnknownEntity::new_unknown());

    let try_scope = self.try_scope();
    self.exit_to(try_scope.cf_scope_depth);
  }

  pub fn forward_throw(&mut self, values: Vec<Entity<'a>>) {
    if values.is_empty() {
      self.may_throw();
    } else {
      let thrown_val =
        UnknownEntity::new_computed_unknown(box_consumable(ConsumableNode::new_box(values)));
      self.explicit_throw_impl(thrown_val);
    }
  }

  fn explicit_throw_impl(&mut self, value: Entity<'a>) {
    let try_scope = self.try_scope();
    let exec_dep = self.get_exec_dep(try_scope.cf_scope_depth);

    let try_scope = self.try_scope_mut();
    try_scope.may_throw = true;
    try_scope.thrown_values.push(ForwardedEntity::new(value, box_consumable(exec_dep)));
  }
}
