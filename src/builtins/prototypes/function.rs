use oxc::{index::Idx, semantic::SymbolId};

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
        // This can be any value
        let arguments_object_id = SymbolId::from_usize(0);
        match arg.test_is_undefined() {
          Some(true) => analyzer.factory.entity(ArrayEntity::new(cf_scope, arguments_object_id)),
          Some(false) => arg,
          None => analyzer.factory.union(vec![
            arg,
            analyzer.factory.entity(ArrayEntity::new(cf_scope, arguments_object_id)),
          ]),
        }
      };
      let this_arg = args.pop().unwrap();
      this.call(analyzer, dep, this_arg, args_arg)
    }),
  );
  prototype.insert(
    "call",
    factory.implemented_builtin_fn(|analyzer, dep, this, args| {
      let (this_arg, args_arg, _deps) = args.destruct_as_array(analyzer, dep.cloned(), 1);
      this.call(analyzer, dep, this_arg[0], args_arg)
    }),
  );
  prototype.insert("bind", factory.pure_fn_returns_unknown);
  // FIXME: Consume self / warn
  prototype.insert("length", factory.unknown_number);
  prototype.insert("arguments", factory.immutable_unknown);
  prototype.insert("caller", factory.immutable_unknown);
  prototype.insert("name", factory.unknown_string);

  prototype
}
