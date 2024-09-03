use crate::entity::{
  entity::Entity,
  literal::LiteralEntity,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use rustc_hash::FxHashMap;

pub fn create_globals<'a>() -> FxHashMap<&'static str, Entity<'a>> {
  let mut globals = FxHashMap::default();

  globals.insert("undefined", LiteralEntity::new_undefined());
  globals.insert("Infinity", LiteralEntity::new_infinity(true));
  globals.insert("NaN", LiteralEntity::new_nan());
  globals.insert("undefined", LiteralEntity::new_undefined());
  globals.insert("eval", UnknownEntity::new(UnknownEntityKind::Function));

  globals
}
