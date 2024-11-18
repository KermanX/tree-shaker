mod context;
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
  entity::{Entity, EntityFactory, ObjectEntity, ObjectPropertyValue},
  init_namespace,
};
use context::{create_react_create_context_impl, create_react_use_context_impl, ReactContexts};
use create_element::create_react_create_element_impl;
use forward_ref::create_react_forward_ref_impl;
use jsx::create_react_jsx_impl;
use jsxs::create_react_jsxs_impl;
use memo::create_react_memo_impl;

#[derive(Debug, Default)]
pub struct AnalyzerDataForReact<'a> {
  pub contexts: ReactContexts<'a>,
}

pub fn create_react_namespace<'a>(
  factory: &'a EntityFactory<'a>,
  prototypes: &'a BuiltinPrototypes<'a>,
) -> Entity<'a> {
  let namespace = ObjectEntity::new_builtin(REACT_NAMESPACE_OBJECT_ID, &prototypes.null, false);
  namespace.init_rest(ObjectPropertyValue::Field(factory.immutable_unknown, true));

  init_namespace!(namespace, {
    "forwardRef" => create_react_forward_ref_impl(factory),
    "memo" => create_react_memo_impl(factory),
    "createElement" => create_react_create_element_impl(factory),
    "createContext" => create_react_create_context_impl(factory),
    "useContext" => create_react_use_context_impl(factory),
  });

  factory.entity(namespace)
}

pub fn create_react_jsx_runtime_namespace<'a>(
  factory: &'a EntityFactory<'a>,
  prototypes: &'a BuiltinPrototypes<'a>,
) -> Entity<'a> {
  let object =
    ObjectEntity::new_builtin(REACT_JSX_RUNTIME_NAMESPACE_OBJECT_ID, &prototypes.null, false);
  object.init_rest(ObjectPropertyValue::Field(factory.immutable_unknown, true));

  init_namespace!(object, {
    "jsx" => create_react_jsx_impl(factory),
    "jsxs" => create_react_jsxs_impl(factory),
  });

  factory.entity(object)
}
