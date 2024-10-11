use super::{object::create_object_prototype, Prototype};
use crate::entity::{ArrayEntity, EntityFactory};

pub fn create_function_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  let mut prototype = create_object_prototype(factory);

  prototype.insert(
    "apply",
    factory.implemented_builtin_fn(|analyzer, dep, this, args| {
      let mut args = args.destruct_as_array(analyzer, dep.cloned(), 2).0;
      let args_arg = {
        let arg = args.pop().unwrap();
        let cf_scope = analyzer.scope_context.cf.current_id();
        let variable_scope = analyzer.scope_context.variable.current_id();
        match arg.test_is_undefined() {
          Some(true) => analyzer.factory.entity(ArrayEntity::new(cf_scope, variable_scope)),
          Some(false) => arg,
          None => analyzer
            .factory
            .union(vec![arg, analyzer.factory.entity(ArrayEntity::new(cf_scope, variable_scope))]),
        }
      };
      let this_arg = args.pop().unwrap();
      this.call(analyzer, dep, this_arg, args_arg)
    }),
  );
  prototype.insert(
    "call",
    factory.implemented_builtin_fn(|analyzer, dep, this, args| {
      let (this_arg, args_arg) = args.destruct_as_array(analyzer, dep.cloned(), 1);
      this.call(analyzer, dep, this_arg[0], args_arg)
    }),
  );
  prototype.insert("bind", factory.pure_fn_returns_unknown);
  // FIXME: Consume self / warn
  prototype.insert("length", factory.unknown_number);
  prototype.insert("arguments", factory.unknown);
  prototype.insert("caller", factory.unknown);
  prototype.insert("name", factory.unknown);

  prototype
}
