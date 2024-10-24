use super::{null::create_null_prototype, Prototype};
use crate::entity::EntityFactory;

pub fn create_object_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  let mut prototype = create_null_prototype(factory);

  prototype.insert("constructor", factory.immutable_unknown);
  prototype.insert("hasOwnProperty", factory.pure_fn_returns_boolean);
  prototype.insert("isPrototypeOf", factory.pure_fn_returns_boolean);
  prototype.insert("propertyIsEnumerable", factory.pure_fn_returns_boolean);
  prototype.insert("toLocaleString", factory.pure_fn_returns_string);
  prototype.insert("toString", factory.pure_fn_returns_string);
  prototype.insert("valueOf", factory.pure_fn_returns_unknown);

  prototype
}
