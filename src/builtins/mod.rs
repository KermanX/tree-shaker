mod globals;
mod import_meta;
mod protos;

use crate::entity::entity::Entity;
use globals::create_globals;
use import_meta::create_import_meta;
pub use protos::Prototype;
use protos::{create_builtin_prototypes, BuiltinPrototypes};
use rustc_hash::FxHashMap;

pub struct Builtins<'a> {
  globals: FxHashMap<&'static str, Entity<'a>>,
  pub prototypes: BuiltinPrototypes<'a>,
  import_meta: Entity<'a>,
}

impl<'a> Builtins<'a> {
  pub fn new() -> Self {
    Self {
      globals: create_globals(),
      prototypes: create_builtin_prototypes(),
      import_meta: create_import_meta(),
    }
  }

  pub fn get_global(&self, name: &str) -> Option<&Entity<'a>> {
    self.globals.get(name)
  }

  pub fn is_global(&self, name: &str) -> bool {
    self.globals.contains_key(name)
  }

  pub fn get_import_meta(&self) -> Entity<'a> {
    self.import_meta.clone()
  }
}
