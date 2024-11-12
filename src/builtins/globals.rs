use crate::{
  config::TreeShakeJsxPreset,
  entity::{Entity, EntityFactory},
  TreeShakeConfig,
};
use rustc_hash::FxHashMap;

use super::{prototypes::BuiltinPrototypes, react::create_react_namespace};

pub fn create_globals<'a>(
  config: &'a TreeShakeConfig,
  factory: &'a EntityFactory<'a>,
  prototypes: &'a BuiltinPrototypes<'a>,
) -> FxHashMap<&'static str, Entity<'a>> {
  let mut globals = FxHashMap::default();

  globals.insert("undefined", factory.undefined);
  globals.insert("Infinity", factory.infinity(true));
  globals.insert("NaN", factory.nan);
  globals.insert("undefined", factory.undefined);
  globals.insert("eval", factory.immutable_unknown);

  if config.jsx == TreeShakeJsxPreset::React {
    globals.insert("React", create_react_namespace(factory, prototypes));
  }

  globals
}
