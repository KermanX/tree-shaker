use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable, ConsumableTrait},
  entity::{Entity, EntityFactory},
  init_object,
};
use oxc::{index::IndexVec, semantic::SymbolId};

#[derive(Debug)]
pub struct ReactContextData<'a> {
  consumed: bool,
  default_value: Entity<'a>,
  stack: Vec<Entity<'a>>,
}

impl<'a> ReactContextData<'a> {
  pub fn get_current(&self, factory: &'a EntityFactory<'a>) -> Entity<'a> {
    if self.consumed {
      factory.unknown()
    } else {
      self.stack.last().copied().unwrap_or(self.default_value)
    }
  }
}

pub type ReactContexts<'a> = IndexVec<SymbolId, ReactContextData<'a>>;

#[derive(Debug, Clone, Copy)]
struct ContextId(SymbolId);

pub fn create_react_create_context_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn(|analyzer, dep, _this, args| {
    let default_value = args.destruct_as_array(analyzer, dep, 1).0[0];

    let context = analyzer.new_empty_object(&analyzer.builtins.prototypes.object);

    let context_id = ContextId(analyzer.builtins.react_data.contexts.push(ReactContextData {
      consumed: false,
      default_value,
      stack: Vec::new(),
    }));

    init_object!(context, {
      "__#internal__context_id" => analyzer.serialize_internal_symbol_id(context_id.0),
      "Provider" => create_react_context_provider_impl(factory, context_id),
      "Consumer" => create_react_context_consumer_impl(factory, context_id),
    });

    factory.entity(context)
  })
}

impl<'a> ConsumableTrait<'a> for ContextId {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    let data = &mut analyzer.builtins.react_data.contexts[self.0];
    data.consumed = true;
  }
  fn cloned(&self) -> Consumable<'a> {
    box_consumable(*self)
  }
}

fn create_react_context_provider_impl<'a>(
  factory: &'a EntityFactory<'a>,
  context_id: ContextId,
) -> Entity<'a> {
  factory.computed(
    factory.implemented_builtin_fn(move |analyzer, dep, _this, args| {
      let props = args.destruct_as_array(analyzer, dep.cloned(), 1).0[0];
      let value = props.get_property(analyzer, dep.cloned(), analyzer.factory.string("value"));

      analyzer.builtins.react_data.contexts[context_id.0].stack.push(value);

      let children = props.get_property(analyzer, dep, analyzer.factory.string("children"));
      children.consume(analyzer);

      analyzer.builtins.react_data.contexts[context_id.0].stack.pop();

      analyzer.factory.immutable_unknown
    }),
    context_id,
  )
}

fn create_react_context_consumer_impl<'a>(
  factory: &'a EntityFactory<'a>,
  context_id: ContextId,
) -> Entity<'a> {
  factory.computed(
    factory.implemented_builtin_fn(move |analyzer, dep, _this, _args| {
      let data = &analyzer.builtins.react_data.contexts[context_id.0];
      let value = data.get_current(factory);
      analyzer.consume(value);
      analyzer.consume(dep);

      analyzer.factory.immutable_unknown
    }),
    context_id,
  )
}

pub fn create_react_use_context_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn(move |analyzer, dep, _this, args| {
    let context_object = args.destruct_as_array(analyzer, box_consumable(()), 1).0[0];
    let context_id = context_object.get_property(
      analyzer,
      box_consumable(()),
      analyzer.factory.string("__#internal__context_id"),
    );
    if let Some(id) = analyzer.parse_internal_symbol_id(context_id) {
      let data = &analyzer.builtins.react_data.contexts[id];
      factory.computed(data.get_current(factory), (context_id, dep))
    } else {
      analyzer.thrown_builtin_error("Invalid React context object");
      factory.unknown()
    }
  })
}
