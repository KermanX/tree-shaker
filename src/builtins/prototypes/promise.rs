use super::{object::create_object_prototype, Prototype};
use crate::entity::EntityFactory;

pub fn create_promise_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  let mut prototype = create_object_prototype(factory);

  prototype.insert("finally", factory.unknown);
  prototype.insert("then", factory.unknown);
  prototype.insert("catch", factory.unknown);

  prototype
}
