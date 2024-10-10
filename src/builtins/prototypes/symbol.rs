use super::{object::create_object_prototype, Prototype};
use crate::entity::EntityFactory;

pub fn create_symbol_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  let mut prototype = create_object_prototype(factory);

  prototype.insert("description", factory.unknown_string);

  prototype
}
