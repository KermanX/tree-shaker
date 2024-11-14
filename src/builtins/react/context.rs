use oxc::semantic::SymbolId;
use crate::{
  consumable::box_consumable,
  entity::{Entity, EntityFactory}, init_namespace,
};

pub fn create_react_create_context_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn(|analyzer, dep, _this, args| {
    let context = analyzer.new_empty_object(&analyzer.builtins.prototypes.object);

    let context_id = analyzer.builtins.react_data.contexts.push(Default::default());

    init_namespace!(context, {
      "Provider" => create_react_context_provider_impl(factory,context_id),
      "Consumer" => create_react_context_consumer_impl(factory,context_id),
    });

    factory.entity(object)
  })
}

fn create_react_context_provider_impl<'a>(factory: &'a EntityFactory<'a>,context_id:SymbolId) -> Entity<'a> {
  factory.implemented_builtin_fn(|analyzer, dep, _this, args| {
    // Called as jsx
    let props = args.destruct_as_array(analyzer, dep.cloned(), 1).0[0];
    let value = props.get_property(analyzer, dep, analyzer.factory.string("value"));
    let children = props.get_property(analyzer, dep, analyzer.factory.string("children"));
  })
}