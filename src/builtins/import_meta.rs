use crate::entity::{
  builtin_fn::PureBuiltinFnEntity,
  entity::Entity,
  object::{ObjectEntity, ObjectProperty, ObjectPropertyValue},
  unknown::{UnknownEntity, UnknownEntityKind},
};

pub fn create_import_meta<'a>() -> Entity<'a> {
  let object = ObjectEntity::default();

  // import.meta.url
  object.string_keyed.borrow_mut().insert(
    "url",
    ObjectProperty {
      definite: true,
      values: vec![ObjectPropertyValue::Property(
        Some(PureBuiltinFnEntity::returns_unknown_entity(UnknownEntityKind::String)),
        None,
      )],
    },
  );

  object.rest.borrow_mut().values.push(ObjectPropertyValue::Property(
    Some(UnknownEntity::new_unknown()),
    Some(UnknownEntity::new_unknown()),
  ));

  Entity::new(object)
}
