use super::{object::create_object_prototype, Prototype};
use crate::entity::{
  array::ArrayEntity,
  builtin_fn::{ImplementedBuiltinFnEntity, PureBuiltinFnEntity},
  entity::Entity,
  union::UnionEntity,
  unknown::{UnknownEntity, UnknownEntityKind},
};

pub fn create_function_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert(
    "apply",
    ImplementedBuiltinFnEntity::new(|analyzer, dep, this, args| {
      let mut args = args.destruct_as_array(analyzer, dep.clone(), 2).0;
      let this_arg = args.pop().unwrap();
      let args_arg = {
        let arg = args.pop().unwrap();
        match arg.test_is_undefined() {
          Some(true) => Entity::new(ArrayEntity::new(vec![], vec![])),
          Some(false) => arg,
          None => UnionEntity::new(vec![arg, Entity::new(ArrayEntity::new(vec![], vec![]))]),
        }
      };
      this.call(analyzer, dep, &this_arg, &args_arg)
    }),
  );
  prototype.insert(
    "call",
    ImplementedBuiltinFnEntity::new(|analyzer, dep, this, args| {
      let (this_arg, args_arg) = args.destruct_as_array(analyzer, dep.clone(), 1);
      this.call(analyzer, dep, &this_arg[0], &args_arg)
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
