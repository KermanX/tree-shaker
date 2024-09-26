use super::{null::create_null_prototype, Prototype};
use crate::entity::builtin_fn::PureBuiltinFnEntity;

pub fn create_object_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_null_prototype();

  prototype.insert("constructor", PureBuiltinFnEntity::returns_object());
  prototype.insert("hasOwnProperty", PureBuiltinFnEntity::returns_boolean());
  prototype.insert("isPrototypeOf", PureBuiltinFnEntity::returns_boolean());
  prototype.insert("propertyIsEnumerable", PureBuiltinFnEntity::returns_boolean());
  prototype.insert("toLocaleString", PureBuiltinFnEntity::returns_string());
  prototype.insert("toString", PureBuiltinFnEntity::returns_string());
  prototype.insert("valueOf", PureBuiltinFnEntity::returns_unknown());

  prototype
}
