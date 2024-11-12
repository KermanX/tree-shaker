mod forward_ref;
mod memo;

use super::{constants::REACT_NAMESPACE_OBJECT_ID, prototypes::BuiltinPrototypes};
use crate::entity::{Entity, EntityFactory, ObjectEntity, ObjectProperty, ObjectPropertyValue};
use forward_ref::create_react_forward_ref_impl;
use memo::create_react_memo_impl;

pub fn create_react_namespace<'a>(
  factory: &'a EntityFactory<'a>,
  prototypes: &'a BuiltinPrototypes<'a>,
) -> Entity<'a> {
  let object = ObjectEntity::new_builtin(REACT_NAMESPACE_OBJECT_ID, &prototypes.null);

  object.string_keyed.borrow_mut().insert(
    "forwardRef",
    ObjectProperty {
      definite: true,
      values: vec![ObjectPropertyValue::Field(create_react_forward_ref_impl(factory))],
    },
  );

  object.string_keyed.borrow_mut().insert(
    "memo",
    ObjectProperty {
      definite: true,
      values: vec![ObjectPropertyValue::Field(create_react_memo_impl(factory))],
    },
  );

  factory.entity(object)
}
