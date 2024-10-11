use crate::entity::{Entity, EntityFactory};
use rustc_hash::FxHashMap;

pub fn create_globals<'a>(factory: &EntityFactory<'a>) -> FxHashMap<&'static str, Entity<'a>> {
  let mut globals = FxHashMap::default();

  globals.insert("undefined", factory.undefined);
  globals.insert("Infinity", factory.infinity(true));
  globals.insert("NaN", factory.nan);
  globals.insert("undefined", factory.undefined);
  globals.insert("eval", factory.unknown_function);

  globals
}
