mod array;
mod bigint;
mod boolean;
mod function;
mod null;
mod number;
mod object;
mod regexp;
mod string;
mod symbol;

use crate::entity::{
  entity::Entity, entry::EntryEntity, literal::LiteralEntity, union::UnionEntity,
  unknown::UnknownEntity,
};
use rustc_hash::FxHashMap;

pub struct Prototype<'a>(FxHashMap<&'static str, Entity<'a>>);

impl<'a> Prototype<'a> {
  pub fn new() -> Self {
    Self(FxHashMap::default())
  }

  pub fn insert(&mut self, key: &'static str, value: Entity<'a>) {
    self.0.insert(key, value);
  }

  pub fn get(&self, key: &str) -> Option<&Entity<'a>> {
    self.0.get(key)
  }

  pub fn get_property(&self, key: &Entity<'a>) -> (bool, Entity<'a>) {
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let mut values = vec![];
      let mut undefined_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(property) = self.get(key) {
              values.push(property.clone());
            } else if !undefined_added {
              undefined_added = true;
              values.push(LiteralEntity::new_undefined());
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!(),
        }
      }
      (false, EntryEntity::new(UnionEntity::new(values), key.clone()))
    } else {
      // TODO: like set_property, call getters and collect all possible values
      (false, EntryEntity::new(UnknownEntity::new_unknown(), key.clone()))
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
    regexp: regexp::create_regexp_prototype(),
    string: string::create_string_prototype(),
    symbol: symbol::create_symbol_prototype(),
  }
}
