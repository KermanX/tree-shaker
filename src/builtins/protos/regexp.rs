use super::{object::create_object_prototype, Prototype};
use crate::entity::builtin_fn::PureBuiltinFnEntity;

pub fn create_regexp_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert("exec", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("test", PureBuiltinFnEntity::returns_boolean());
  prototype.insert("toString", PureBuiltinFnEntity::returns_string());

  prototype
}
