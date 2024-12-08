use crate::{
  consumable::box_consumable,
  entity::{Entity, EntityFactory},
};

pub fn create_react_jsxs_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn("React::jsxs", |analyzer, dep, _this, args| {
    let args = args.destruct_as_array(analyzer, dep, 3).0;
    let [tag, props, key] = args[..] else { unreachable!() };
    let destructable_dep = props.get_destructable(analyzer, box_consumable(()));
    analyzer.consume(destructable_dep);
    props.set_property(analyzer, box_consumable(()), analyzer.factory.string("key"), key);
    analyzer.factory.react_element(tag, props)
  })
}
