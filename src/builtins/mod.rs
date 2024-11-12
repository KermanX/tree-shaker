mod constants;
mod globals;
mod import_meta;
mod prototypes;
mod react;

use crate::{
  entity::{Entity, EntityFactory},
  TreeShakeConfig,
};
use prototypes::BuiltinPrototypes;
pub use prototypes::Prototype;
use rustc_hash::FxHashMap;

pub struct Builtins<'a> {
  pub config: &'a TreeShakeConfig,
  pub factory: &'a EntityFactory<'a>,

  pub prototypes: &'a BuiltinPrototypes<'a>,
  pub globals: FxHashMap<&'static str, Entity<'a>>,
  pub import_meta: Entity<'a>,
}

impl<'a> Builtins<'a> {
  pub fn new(config: &'a TreeShakeConfig, factory: &'a EntityFactory<'a>) -> Self {
    let prototypes = Self::create_builtin_prototypes(factory);
    let mut builtins = Self {
      config,
      factory,

      prototypes,
      import_meta: Self::create_import_meta(factory, prototypes),
      globals: Default::default(), // Initialize later
    };
    builtins.init_globals();
    builtins
  }
}
