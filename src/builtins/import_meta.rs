use crate::entity::{
  Entity, ImplementedBuiltinFnEntity, ObjectEntity, ObjectProperty, ObjectPropertyValue,
  UnknownEntity,
};

pub fn create_import_meta<'a>() -> Entity<'a> {
  let object = ObjectEntity::new();

  // import.meta.url
  object.string_keyed.borrow_mut().insert(
    "url",
    ObjectProperty {
      definite: true,
      values: vec![ObjectPropertyValue::Property(
        Some(ImplementedBuiltinFnEntity::new(|_, _, _, _| UnknownEntity::new_string()).into()),
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
