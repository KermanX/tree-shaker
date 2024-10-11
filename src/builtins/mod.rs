mod globals;
mod import_meta;
mod prototypes;

use crate::entity::{Entity, EntityFactory};
use globals::create_globals;
use import_meta::create_import_meta;
pub use prototypes::Prototype;
use prototypes::{create_builtin_prototypes, BuiltinPrototypes};
use rustc_hash::FxHashMap;

pub struct Builtins<'a> {
  pub globals: FxHashMap<&'static str, Entity<'a>>,
  pub prototypes: &'a BuiltinPrototypes<'a>,
  pub import_meta: Entity<'a>,
}

impl<'a> Builtins<'a> {
  pub fn new(factory: &EntityFactory<'a>) -> Self {
    Self {
      globals: create_globals(factory),
      prototypes: factory.allocator.alloc(create_builtin_prototypes(factory)),
      import_meta: create_import_meta(factory),
    }
  }

  pub fn get_global(&self, name: &str) -> Option<Entity<'a>> {
    self.globals.get(name).copied()
  }

  pub fn is_global(&self, name: &str) -> bool {
    self.globals.contains_key(name)
  }

  pub fn get_import_meta(&self) -> Entity<'a> {
    self.import_meta.clone()
  }
}
