use super::constants::IMPORT_META_OBJECT_ID;
use crate::entity::{Entity, EntityFactory, ObjectEntity, ObjectProperty, ObjectPropertyValue};

pub fn create_import_meta<'a>(factory: &EntityFactory<'a>) -> Entity<'a> {
  let object = ObjectEntity::new_builtin(IMPORT_META_OBJECT_ID);

  // import.meta.url
  object.string_keyed.borrow_mut().insert(
    "url",
    ObjectProperty {
      definite: true,
      values: vec![ObjectPropertyValue::Property(
        Some(factory.implemented_builtin_fn(|analyzer, _, _, _| analyzer.factory.unknown_string)),
        None,
      )],
    },
  );

  object
    .rest
    .borrow_mut()
    .values
    .push(ObjectPropertyValue::Property(Some(factory.unknown), Some(factory.unknown)));

  factory.entity(object)
}
