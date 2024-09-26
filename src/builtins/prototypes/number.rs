use super::{object::create_object_prototype, Prototype};
use crate::entity::PureBuiltinFnEntity;

pub fn create_number_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert("toExponential", PureBuiltinFnEntity::returns_string());
  prototype.insert("toFixed", PureBuiltinFnEntity::returns_string());
  prototype.insert("toLocaleString", PureBuiltinFnEntity::returns_string());
  prototype.insert("toPrecision", PureBuiltinFnEntity::returns_string());
  prototype.insert("valueOf", PureBuiltinFnEntity::returns_number());

  prototype
}
