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
  consumable::{box_consumable, Consumable},
  entity::{Entity, LiteralEntity, UnionEntity, UnknownEntity},
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

  pub fn get(&self, key: &str) -> Option<&Entity<'a>> {
    self.0.get(key)
  }

  pub fn get_property(&self, rc: &Entity<'a>, key: &Entity<'a>, dep: Consumable<'a>) -> Entity<'a> {
    let dep = box_consumable((dep, rc.clone(), key.clone()));
    'known: {
      if let Some(key_literals) = key.get_to_literals() {
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
        return UnionEntity::new_computed(values, dep);
      }
    }
    UnknownEntity::new_computed_unknown(dep)
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

pub fn create_builtin_prototypes<'a>() -> BuiltinPrototypes<'a> {
  BuiltinPrototypes {
    array: array::create_array_prototype(),
    bigint: bigint::create_bigint_prototype(),
    boolean: boolean::create_boolean_prototype(),
    function: function::create_function_prototype(),
    null: null::create_null_prototype(),
    number: number::create_number_prototype(),
    object: object::create_object_prototype(),
    promise: promise::create_promise_prototype(),
    regexp: regexp::create_regexp_prototype(),
    string: string::create_string_prototype(),
    symbol: symbol::create_symbol_prototype(),
  }
}
