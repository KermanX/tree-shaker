mod array;
mod bigint;
mod boolean;
mod function;
mod null;
mod number;
mod object;
mod promise;
mod regexp;
mod string;
mod symbol;

use crate::{
  analyzer::Analyzer,
  entity::{Consumable, Entity, EntityFactory, LiteralEntity},
};
use rustc_hash::FxHashMap;

pub struct Prototype<'a>(FxHashMap<&'static str, Entity<'a>>);

impl<'a> Prototype<'a> {
  pub fn new() -> Self {
    Self(FxHashMap::default())
  }

  pub fn insert(&mut self, key: &'static str, value: impl Into<Entity<'a>>) {
    self.0.insert(key, value.into());
  }

  pub fn get(&self, key: &str) -> Option<Entity<'a>> {
    self.0.get(key).copied()
  }

  pub fn get_property(
    &self,
    analyzer: &Analyzer<'a>,
    rc: Entity<'a>,
    key: Entity<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    let key = key.get_to_property_key(analyzer);
    'known: {
      if let Some(key_literals) = key.get_to_literals(analyzer) {
        let mut values = vec![];
        for key_literal in key_literals {
          match key_literal {
            LiteralEntity::String(key) => {
              if let Some(property) = self.get(key) {
                values.push(property.clone());
              } else {
                break 'known;
              }
            }
            LiteralEntity::Symbol(_, _) => break 'known,
            _ => unreachable!(),
          }
        }
        return analyzer.factory.new_computed_union(values, (dep, rc.clone(), key.clone()));
      }
    }
    analyzer.factory.new_computed_unknown((dep, rc.clone(), key.clone()))
  }
}

pub struct BuiltinPrototypes<'a> {
  pub array: Prototype<'a>,
  pub bigint: Prototype<'a>,
  pub boolean: Prototype<'a>,
  pub function: Prototype<'a>,
  pub null: Prototype<'a>,
  pub number: Prototype<'a>,
  pub object: Prototype<'a>,
  pub promise: Prototype<'a>,
  pub regexp: Prototype<'a>,
  pub string: Prototype<'a>,
  pub symbol: Prototype<'a>,
}

pub fn create_builtin_prototypes<'a>(factory: &EntityFactory<'a>) -> BuiltinPrototypes<'a> {
  BuiltinPrototypes {
    array: array::create_array_prototype(factory),
    bigint: bigint::create_bigint_prototype(factory),
    boolean: boolean::create_boolean_prototype(factory),
    function: function::create_function_prototype(factory),
    null: null::create_null_prototype(factory),
    number: number::create_number_prototype(factory),
    object: object::create_object_prototype(factory),
    promise: promise::create_promise_prototype(factory),
    regexp: regexp::create_regexp_prototype(factory),
    string: string::create_string_prototype(factory),
    symbol: symbol::create_symbol_prototype(factory),
  }
}
