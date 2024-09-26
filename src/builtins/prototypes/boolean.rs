use super::{object::create_object_prototype, Prototype};
use crate::entity::PureBuiltinFnEntity;

pub fn create_boolean_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert("valueOf", PureBuiltinFnEntity::returns_boolean());

  prototype
}
