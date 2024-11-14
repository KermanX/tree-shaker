mod create_element;
mod forward_ref;
mod jsx;
mod jsxs;
mod memo;

use super::{
  constants::{REACT_JSX_RUNTIME_NAMESPACE_OBJECT_ID, REACT_NAMESPACE_OBJECT_ID},
  prototypes::BuiltinPrototypes,
};
use crate::{
  entity::{Entity, EntityFactory, ObjectEntity, ObjectProperty, ObjectPropertyValue},
  init_namespace,
};
use create_element::create_react_create_element_impl;
use forward_ref::create_react_forward_ref_impl;
use jsx::create_react_jsx_impl;
use jsxs::create_react_jsxs_impl;
use memo::create_react_memo_impl;

pub fn create_react_namespace<'a>(
  factory: &'a EntityFactory<'a>,
  prototypes: &'a BuiltinPrototypes<'a>,
) -> Entity<'a> {
  let mut namespace = ObjectEntity::new_builtin(REACT_NAMESPACE_OBJECT_ID, &prototypes.null);
  namespace
    .rest
    .borrow_mut()
    .values
    .push(ObjectPropertyValue::Field(factory.immutable_unknown, Some(true)));
  namespace.consumable = false;

  init_namespace!(namespace, {
    "forwardRef" => create_react_forward_ref_impl(factory),
    "memo" => create_react_memo_impl(factory),
    "createElement" => create_react_create_element_impl(factory),
  });

  factory.entity(namespace)
}

pub fn create_react_jsx_runtime_namespace<'a>(
  factory: &'a EntityFactory<'a>,
  prototypes: &'a BuiltinPrototypes<'a>,
) -> Entity<'a> {
  let mut object =
    ObjectEntity::new_builtin(REACT_JSX_RUNTIME_NAMESPACE_OBJECT_ID, &prototypes.null);
  object
    .rest
    .borrow_mut()
    .values
    .push(ObjectPropertyValue::Field(factory.immutable_unknown, Some(true)));
  object.consumable = false;

  init_namespace!(object, {
    "jsx" => create_react_jsx_impl(factory),
    "jsxs" => create_react_jsxs_impl(factory),
  });

  factory.entity(object)
}
