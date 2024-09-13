use crate::{
  analyzer::Analyzer,
  entity::{entity::Entity, unknown::UnknownEntity},
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

  pub fn throw(&mut self, value: Entity<'a>) {
    self.thrown_values.push(value);
    self.may_throw = true;
  }

  pub fn thrown_val(self) -> Entity<'a> {
    // Always unknown here
    UnknownEntity::new_unknown_with_deps(self.thrown_values)
  }
}

impl<'a> Analyzer<'a> {
  pub fn may_throw(&mut self) {
    self.try_scope_mut().may_throw = true;
  }
}
