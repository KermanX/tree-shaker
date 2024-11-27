use crate::{
  builtins::prototypes::BuiltinPrototypes,
  consumable::ConsumableNode,
  entity::{Entity, EntityFactory, TypeofResult},
};

pub fn create_class_names_namespace<'a>(
  factory: &'a EntityFactory<'a>,
  _prototypes: &'a BuiltinPrototypes<'a>,
) -> Entity<'a> {
  factory.implemented_builtin_fn("classnames::default", |analyzer, dep, _this, args| {
    let (class_names, rest, iterate_dep) = args.iterate(analyzer, dep.cloned());

    let mut deps_1 = vec![];
    let mut deps_2 = vec![iterate_dep];
    for class_name in class_names {
      if TypeofResult::Object.contains(class_name.test_typeof(analyzer)) {
        // This may be an array. However, this makes no difference in this logic.
        let (properties, enumerate_dep) = class_name.enumerate_properties(analyzer, dep.cloned());
        deps_2.push(enumerate_dep);
        for (_, key, value) in properties {
          if value.test_truthy(analyzer) != Some(false) {
            deps_1.push(key);
            deps_1.push(value);
          }
        }
      } else {
        deps_1.push(class_name);
      }
    }

    analyzer.factory.computed_unknown_string((
      ConsumableNode::new(deps_1),
      ConsumableNode::new(deps_2),
      rest,
    ))
  })
}
