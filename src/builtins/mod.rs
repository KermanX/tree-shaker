use crate::entity::entity::Entity;
use globals::create_globals;
use rustc_hash::FxHashMap;
mod globals;

pub(crate) struct Builtins<'a> {
  globals: FxHashMap<&'static str, Entity<'a>>,
}

impl<'a> Builtins<'a> {
  pub fn new() -> Self {
    Self { globals: create_globals() }
  }

  pub fn get_global(&self, name: &str) -> Option<&Entity<'a>> {
    self.globals.get(name)
  }

  pub fn is_global(&self, name: &str) -> bool {
    self.globals.contains_key(name)
  }
}
