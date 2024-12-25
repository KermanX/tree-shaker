use crate::{
  consumable::box_consumable,
  entity::{Entity, EntityFactory},
};

pub fn create_react_forward_ref_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn("React::forwardRef", |analyzer, dep, _this, args| {
    let renderer = args.destruct_as_array(analyzer, dep, 1).0[0];

    analyzer.dynamic_implemented_builtin(
      "React::ForwardRefReturn",
      move |analyzer, dep, this, args| {
        let props = args.destruct_as_array(analyzer, box_consumable(()), 1).0[0];
        let r#ref = analyzer.factory.unknown();

        renderer.call(
          analyzer,
          dep,
          this,
          analyzer.factory.arguments(vec![(false, props), (false, r#ref)]),
        )
      },
    )
  })
}
