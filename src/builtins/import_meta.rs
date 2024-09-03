use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};

use crate::entity::{
  builtin_fn::BuiltinFnEntity,
  entity::Entity,
  object::{ObjectEntity, ObjectProperty, ObjectPropertyValue},
  unknown::{UnknownEntity, UnknownEntityKind},
};

pub fn create_import_meta<'a>() -> Entity<'a> {
  let mut string_keyed = FxHashMap::default();

  // import.meta.url
  string_keyed.insert(
    "url",
    ObjectProperty {
      definite: true,
      values: vec![ObjectPropertyValue::Property(
        Some(BuiltinFnEntity::new(|_, _, _| {
          (false, UnknownEntity::new(UnknownEntityKind::String))
        })),
        None,
      )],
    },
  );

  Rc::new(ObjectEntity {
    scope_path: vec![],
    string_keyed: RefCell::new(string_keyed),
    unknown_keyed: RefCell::new(ObjectProperty::default()),
    rest: RefCell::new(ObjectProperty {
      definite: false,
      values: vec![ObjectPropertyValue::Property(
        Some(UnknownEntity::new_unknown()),
        Some(UnknownEntity::new_unknown()),
      )],
    }),
  })
}
