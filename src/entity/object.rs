use std::{cell::LazyCell, rc::Rc};

use super::EntityValue;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub struct ObjectEntity {
  string_keyed: FxHashMap<String, Rc<EntityValue>>,
  symbol_keyed: FxHashMap<usize, Rc<EntityValue>>,
  pub rest: Rc<EntityValue>,
}

impl Default for ObjectEntity {
  fn default() -> Self {
    ObjectEntity {
      string_keyed: FxHashMap::default(),
      symbol_keyed: FxHashMap::default(),
      rest: Rc::new(EntityValue::Undefined),
    }
  }
}

impl ObjectEntity {
  pub fn init_property(&mut self, key: &EntityValue, value: EntityValue) {
    match key.to_property_key() {
      EntityValue::StringLiteral(key) => {
        self.string_keyed.insert(key, Rc::new(value));
      }
      EntityValue::Symbol(key) => {
        self.symbol_keyed.insert(key.id, Rc::new(value));
      }
      _ => {
        // TODO:
        self.rest = match self.rest.as_ref() {
          EntityValue::Union(values) => {
            let mut values = values.clone();
            values.push(Rc::new(value));
            Rc::new(EntityValue::Union(values))
          }
          _ => Rc::new(EntityValue::Union(vec![Rc::new(value), self.rest.clone()])),
        }
      }
    }
  }

  pub fn get_property(&self, key: &EntityValue) -> Rc<EntityValue> {
    // TODO: builtin properties
    match key.to_property_key() {
      EntityValue::StringLiteral(key) => {
        self.string_keyed.get(&key).map_or_else(|| self.rest.clone(), Rc::clone)
      }
      EntityValue::UnknownString => {
        let mut values: Vec<Rc<EntityValue>> =
          self.string_keyed.values().map(|v| v.clone()).collect();
        values.push(self.rest.clone());
        Rc::new(EntityValue::Union(values).simplify())
      }
      EntityValue::Symbol(key) => {
        self.symbol_keyed.get(&key.id).map_or_else(|| self.rest.clone(), Rc::clone)
      }
      EntityValue::UnknownSymbol => {
        // TODO:
        Rc::new(EntityValue::Unknown)
      }
      EntityValue::Union(keys) => Rc::new(EntityValue::Union(
        keys.iter().map(|key| self.get_property(key)).collect::<Vec<Rc<EntityValue>>>(),
      )),
      _ => unreachable!(),
    }
  }
}

pub const UNKNOWN_OBJECT: LazyCell<ObjectEntity> = LazyCell::new(|| ObjectEntity::default());
