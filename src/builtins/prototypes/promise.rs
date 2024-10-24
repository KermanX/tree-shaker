use super::{object::create_object_prototype, Prototype};
use crate::entity::EntityFactory;

pub fn create_promise_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  let mut prototype = create_object_prototype(factory);

  prototype.insert("finally", factory.immutable_unknown);
  prototype.insert("then", factory.immutable_unknown);
  prototype.insert("catch", factory.immutable_unknown);

  prototype
}
