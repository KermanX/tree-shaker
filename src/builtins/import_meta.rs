use crate::entity::{
  builtin_fn::PureBuiltinFnEntity,
  entity::Entity,
  object::{ObjectEntity, ObjectProperty, ObjectPropertyValue},
  unknown::{UnknownEntity, UnknownEntityKind},
};
use rustc_hash::FxHashMap;
use std::cell::{Cell, RefCell};

pub fn create_import_meta<'a>() -> Entity<'a> {
  let mut string_keyed = FxHashMap::default();

  // import.meta.url
  string_keyed.insert(
    "url",
    ObjectProperty {
      definite: true,
      values: vec![ObjectPropertyValue::Property(
        Some(PureBuiltinFnEntity::returns_unknown_entity(UnknownEntityKind::String)),
        None,
      )],
    },
  );

  Entity::new(ObjectEntity {
    consumed: Cell::new(false),
    cf_scopes: vec![],
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
