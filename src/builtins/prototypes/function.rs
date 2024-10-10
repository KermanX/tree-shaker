use super::{object::create_object_prototype, Prototype};
use crate::entity::{
  ArrayEntity, Entity, ImplementedBuiltinFnEntity, PureBuiltinFnEntity, UnionEntity, UnknownEntity,
};

pub fn create_function_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert(
    "apply",
    ImplementedBuiltinFnEntity::new(|analyzer, dep, this, args| {
      let mut args = args.destruct_as_array(analyzer, dep.cloned(), 2).0;
      let args_arg = {
        let arg = args.pop().unwrap();
        let cf_scope = analyzer.scope_context.cf.current_id();
        let variable_scope = analyzer.scope_context.variable.current_id();
        match arg.test_is_undefined() {
          Some(true) => Entity::new(ArrayEntity::new(cf_scope, variable_scope)),
          Some(false) => arg,
          None => {
            UnionEntity::new(vec![arg, Entity::new(ArrayEntity::new(cf_scope, variable_scope))])
          }
        }
      };
      let this_arg = args.pop().unwrap();
      this.call(analyzer, dep, &this_arg, &args_arg)
    }),
  );
  prototype.insert(
    "call",
    ImplementedBuiltinFnEntity::new(|analyzer, dep, this, args| {
      let (this_arg, args_arg) = args.destruct_as_array(analyzer, dep.cloned(), 1);
      this.call(analyzer, dep, &this_arg[0], &args_arg)
    }),
  );
  prototype.insert("bind", PureBuiltinFnEntity::returns_unknown());
  // FIXME: Consume self / warn
  prototype.insert("length", UnknownEntity::new_number());
  prototype.insert("arguments", UnknownEntity::new_unknown());
  prototype.insert("caller", UnknownEntity::new_unknown());
  prototype.insert("name", UnknownEntity::new_unknown());

  prototype
}
