use super::{object::create_object_prototype, Prototype};
use crate::entity::EntityFactory;

pub fn create_boolean_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  let mut prototype = create_object_prototype(factory);

  prototype.insert("valueOf", factory.pure_fn_returns_boolean);

  prototype
}
