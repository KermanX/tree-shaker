use super::{object::create_object_prototype, Prototype};
use crate::entity::EntityFactory;

pub fn create_regexp_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  let mut prototype = create_object_prototype(factory);

  prototype.insert("exec", factory.pure_fn_returns_unknown);
  prototype.insert("test", factory.pure_fn_returns_boolean);
  prototype.insert("toString", factory.pure_fn_returns_string);

  prototype
}
