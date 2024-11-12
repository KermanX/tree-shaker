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
mod utils;

use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable},
  entity::{Entity, EntityFactory, LiteralEntity},
};
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;

use super::Builtins;

#[derive(Default)]
pub struct Prototype<'a> {
  string_keyed: FxHashMap<&'static str, Entity<'a>>,
  symbol_keyed: FxHashMap<SymbolId, Entity<'a>>,
}

impl<'a> Prototype<'a> {
  pub fn insert_string_keyed(&mut self, key: &'static str, value: impl Into<Entity<'a>>) {
    self.string_keyed.insert(key, value.into());
  }

  pub fn insert_symbol_keyed(&mut self, key: SymbolId, value: impl Into<Entity<'a>>) {
    self.symbol_keyed.insert(key, value.into());
  }

  pub fn get_string_keyed(&self, key: &str) -> Option<Entity<'a>> {
    self.string_keyed.get(key).copied()
  }

  pub fn get_symbol_keyed(&self, key: SymbolId) -> Option<Entity<'a>> {
    self.symbol_keyed.get(&key).copied()
  }

  pub fn get_literal_keyed(&self, key: LiteralEntity) -> Option<Entity<'a>> {
    match key {
      LiteralEntity::String(key) => self.get_string_keyed(key),
      LiteralEntity::Symbol(key, _) => self.get_symbol_keyed(key),
      _ => unreachable!(),
    }
  }

  pub fn get_property(
    &self,
    analyzer: &Analyzer<'a>,
    rc: Entity<'a>,
    key: Entity<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    let key = key.get_to_property_key(analyzer);
    let dep = box_consumable((dep, rc.clone(), key.clone()));
    if let Some(key_literals) = key.get_to_literals(analyzer) {
      let mut values = vec![];
      let mut undefined_added = false;
      for key_literal in key_literals {
        if let Some(property) = self.get_literal_keyed(key_literal) {
          values.push(property);
        } else if !undefined_added {
          undefined_added = true;
          values.push(analyzer.factory.undefined);
        }
      }
      analyzer.factory.computed_union(values, dep)
    } else {
      analyzer.factory.computed_unknown(dep)
    }
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

impl<'a> Builtins<'a> {
  pub fn create_builtin_prototypes(factory: &EntityFactory<'a>) -> &'a BuiltinPrototypes<'a> {
    factory.alloc(BuiltinPrototypes {
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
    })
  }
}
