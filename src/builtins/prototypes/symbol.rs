use super::{object::create_object_prototype, Prototype};
use crate::entity::unknown::UnknownEntity;

pub fn create_symbol_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert("description", UnknownEntity::new_string());

  prototype
}
