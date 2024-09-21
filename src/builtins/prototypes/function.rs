use super::{object::create_object_prototype, Prototype};
use crate::entity::{
  array::ArrayEntity,
  builtin_fn::{ImplementedBuiltinFnEntity, PureBuiltinFnEntity},
  entity::Entity,
  unknown::{UnknownEntity, UnknownEntityKind},
};

pub fn create_function_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert(
    "apply",
    ImplementedBuiltinFnEntity::new(|analyzer, dep, this, args| {
      let args = args
        .destruct_as_array(analyzer, dep.clone(), 1)
        .0
        .pop()
        .unwrap_or_else(|| Entity::new(ArrayEntity::new(vec![], vec![])));
      this.call(analyzer, dep, this, &args)
    }),
  );
  prototype.insert(
    "call",
    ImplementedBuiltinFnEntity::new(|analyzer, dep, this, args| {
      this.call(analyzer, dep, this, args)
    }),
  );
  prototype.insert("bind", PureBuiltinFnEntity::returns_unknown());
  // FIXME: Consume self / warn
  prototype.insert("length", UnknownEntity::new(UnknownEntityKind::Number));
  prototype.insert("arguments", UnknownEntity::new_unknown());
  prototype.insert("caller", UnknownEntity::new_unknown());
  prototype.insert("name", UnknownEntity::new_unknown());

  prototype
}
