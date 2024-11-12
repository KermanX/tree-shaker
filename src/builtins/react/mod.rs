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
  object
    .rest
    .borrow_mut()
    .values
    .push(ObjectPropertyValue::Field(factory.immutable_unknown, Some(true)));

  object.string_keyed.borrow_mut().insert(
    "forwardRef",
    ObjectProperty {
      definite: true,
      values: vec![ObjectPropertyValue::Field(create_react_forward_ref_impl(factory), Some(true))],
    },
  );

  object.string_keyed.borrow_mut().insert(
    "memo",
    ObjectProperty {
      definite: true,
      values: vec![ObjectPropertyValue::Field(create_react_memo_impl(factory), Some(true))],
    },
  );

  factory.entity(object)
}
