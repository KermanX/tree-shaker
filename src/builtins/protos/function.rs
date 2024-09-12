use crate::entity::{
  builtin_fn::PureBuiltinFnEntity,
  unknown::{UnknownEntity, UnknownEntityKind},
};

use super::{object::create_object_prototype, Prototype};

pub fn create_function_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert("apply", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("bind", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("call", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("length", UnknownEntity::new(UnknownEntityKind::Number));
  prototype.insert("arguments", UnknownEntity::new_unknown());
  prototype.insert("caller", UnknownEntity::new_unknown());

  prototype
}
