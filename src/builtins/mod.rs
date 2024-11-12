mod constants;
mod globals;
mod import_meta;
mod prototypes;
mod react;

use crate::{
  entity::{Entity, EntityFactory},
  TreeShakeConfig,
};
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
  pub fn new(config: &'a TreeShakeConfig, factory: &'a EntityFactory<'a>) -> Self {
    let prototypes = factory.alloc(create_builtin_prototypes(factory));
    Self {
      prototypes,
      globals: create_globals(config, factory, prototypes),
      import_meta: create_import_meta(factory, prototypes),
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
