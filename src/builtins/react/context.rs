use std::mem;

use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable, ConsumableTrait},
  entity::{Entity, EntityFactory},
  init_object,
};
use oxc::{
  index::{Idx, IndexVec},
  semantic::SymbolId,
};

#[derive(Debug)]
pub struct ReactContextData<'a> {
  object_id: SymbolId,
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContextId(SymbolId);

impl Idx for ContextId {
  fn from_usize(index: usize) -> Self {
    Self(SymbolId::from_usize(index))
  }
  fn index(self) -> usize {
    self.0.index()
  }
}

pub type ReactContexts<'a> = IndexVec<ContextId, ReactContextData<'a>>;

pub fn create_react_create_context_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn("React::createContext", |analyzer, dep, _this, args| {
    let default_value = args.destruct_as_array(analyzer, dep, 1).0[0];

    let context = analyzer.new_empty_object(&analyzer.builtins.prototypes.object);

    let context_id = analyzer.builtins.react_data.contexts.push(ReactContextData {
      object_id: context.object_id,
      consumed: false,
      default_value,
      stack: Vec::new(),
    });

    init_object!(context, {
      "__#internal__consumed_hook" => analyzer.factory.computed_unknown(context_id),
      "__#internal__context_id" => analyzer.serialize_internal_id(context_id.0),
      "Provider" => create_react_context_provider_impl(analyzer, context_id),
      "Consumer" => create_react_context_consumer_impl(analyzer, context_id),
    });

    factory.entity(context)
  })
}

impl<'a> ConsumableTrait<'a> for ContextId {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    let data = &mut analyzer.builtins.react_data.contexts[*self];
    data.consumed = true;
    let default_value = data.default_value;
    let stack = mem::take(&mut data.stack);
    analyzer.consume(default_value);
    analyzer.consume(stack);
  }
  fn cloned(&self) -> Consumable<'a> {
    box_consumable(*self)
  }
}

fn create_react_context_provider_impl<'a>(
  analyzer: &mut Analyzer<'a>,
  context_id: ContextId,
) -> Entity<'a> {
  analyzer.dynamic_implemented_builtin(
    "React::Context::Provider",
    move |analyzer, dep, _this, args| {
      let props = args.destruct_as_array(analyzer, dep.cloned(), 1).0[0];
      let value = props.get_property(analyzer, dep.cloned(), analyzer.factory.string("value"));

      let data = &mut analyzer.builtins.react_data.contexts[context_id];
      let mut need_pop = false;
      if data.consumed {
        analyzer.consume(value);
      } else {
        data.stack.push(analyzer.factory.computed_unknown(value));

        let object_id = data.object_id;
        let should_consume = analyzer
          .add_exhaustive_callbacks(true, (analyzer.scope_context.object_scope_id, object_id));

        if should_consume {
          analyzer.consume(context_id);
        } else {
          let data = &mut analyzer.builtins.react_data.contexts[context_id];
          data.stack.pop();

          data.stack.push(value);
          need_pop = true;
        }
      }

      let children = props.get_property(analyzer, dep, analyzer.factory.string("children"));
      children.consume(analyzer);

      if need_pop {
        analyzer.builtins.react_data.contexts[context_id].stack.pop();
      }

      analyzer.factory.immutable_unknown
    },
  )
}

fn create_react_context_consumer_impl<'a>(
  analyzer: &mut Analyzer<'a>,
  context_id: ContextId,
) -> Entity<'a> {
  analyzer.dynamic_implemented_builtin(
    "React::Context::Consumer",
    move |analyzer, dep, _this, _args| {
      analyzer.consume(dep);
      let data = &analyzer.builtins.react_data.contexts[context_id];
      let value = data.get_current(analyzer.factory);
      analyzer.consume(value);

      analyzer.factory.immutable_unknown
    },
  )
}

pub fn create_react_use_context_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn("React::useContext", move |analyzer, dep, _this, args| {
    let context_object = args.destruct_as_array(analyzer, box_consumable(()), 1).0[0];
    let context_id = context_object.get_property(
      analyzer,
      box_consumable(()),
      analyzer.factory.string("__#internal__context_id"),
    );
    if let Some(id) = analyzer.parse_internal_symbol_id::<ContextId>(context_id) {
      let data = &analyzer.builtins.react_data.contexts[id];
      factory.computed(data.get_current(factory), (context_id, dep))
    } else {
      factory.computed_unknown((context_object, dep))
    }
  })
}
