use crate::{
  analyzer::Analyzer,
  entity::{
    consumable::Consumable, entity::Entity, forwarded::ForwardedEntity, unknown::UnknownEntity,
  },
};

#[derive(Debug)]
pub struct TryScope<'a> {
  pub may_throw: bool,
  pub thrown_values: Vec<Entity<'a>>,
  pub cf_scope_index: usize,
}

impl<'a> TryScope<'a> {
  pub fn new(cf_scope_index: usize) -> Self {
    TryScope { may_throw: false, thrown_values: Vec::new(), cf_scope_index }
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
    // let cf_scope_index = try_scope.cf_scope_index;
    // self.exit_to(cf_scope_index, false);
  }

  pub fn explicit_throw(&mut self, value: Entity<'a>) {
    let try_scope = self.try_scope_mut();

    try_scope.may_throw = true;
    try_scope.thrown_values.push(value);

    let cf_scope_index = try_scope.cf_scope_index;
    self.exit_to(cf_scope_index);
  }

  pub fn explicit_throw_unknown(&mut self) {
    let try_scope = self.try_scope_mut();

    try_scope.may_throw = true;
    if try_scope.thrown_values.is_empty() {
      try_scope.thrown_values.push(UnknownEntity::new_unknown());
    }

    let cf_scope_index = try_scope.cf_scope_index;
    self.exit_to(cf_scope_index);
  }

  pub fn forward_throw(&mut self, values: Vec<Entity<'a>>, dep: impl Into<Consumable<'a>>) {
    if values.is_empty() {
      self.may_throw();
    } else {
      let thrown_val = UnknownEntity::new_computed_unknown(values);
      self.explicit_throw(ForwardedEntity::new(thrown_val, dep.into()));
    }
  }
}
