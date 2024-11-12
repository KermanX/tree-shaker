use super::{constants::IMPORT_META_OBJECT_ID, prototypes::BuiltinPrototypes};
use crate::entity::{Entity, EntityFactory, ObjectEntity, ObjectProperty, ObjectPropertyValue};

pub fn create_import_meta<'a>(
  factory: &'a EntityFactory<'a>,
  prototypes: &'a BuiltinPrototypes<'a>,
) -> Entity<'a> {
  let object = ObjectEntity::new_builtin(IMPORT_META_OBJECT_ID, &prototypes.null);
  object.init_unknown_rest(factory);

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

  factory.entity(object)
}
