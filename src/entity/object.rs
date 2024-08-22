use std::{cell::LazyCell, rc::Rc};

use super::Entity;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub struct ObjectEntity {
  string_keyed: FxHashMap<String, Rc<Entity>>,
  symbol_keyed: FxHashMap<usize, Rc<Entity>>,
  pub rest: Rc<Entity>,
}

impl Default for ObjectEntity {
  fn default() -> Self {
    ObjectEntity {
      string_keyed: FxHashMap::default(),
      symbol_keyed: FxHashMap::default(),
      rest: Rc::new(Entity::Undefined),
    }
  }
}

impl ObjectEntity {
  pub fn init_property(&mut self, key: &Entity, value: Entity) {
    match key.to_property_key() {
      Entity::StringLiteral(key) => {
        self.string_keyed.insert(key, Rc::new(value));
      }
      Entity::Symbol(key) => {
        self.symbol_keyed.insert(key.id, Rc::new(value));
      }
      _ => {
        // TODO:
        self.rest = match self.rest.as_ref() {
          Entity::Union(values) => {
            let mut values = values.clone();
            values.push(Rc::new(value));
            Rc::new(Entity::Union(values))
          }
          _ => Rc::new(Entity::Union(vec![Rc::new(value), self.rest.clone()])),
        }
      }
    }
  }

  pub fn get_property(&self, key: &Entity) -> Rc<Entity> {
    // TODO: builtin properties
    match key.to_property_key() {
      Entity::StringLiteral(key) => {
        self.string_keyed.get(&key).map_or_else(|| self.rest.clone(), Rc::clone)
      }
      Entity::UnknownString => {
        let mut values: Vec<Rc<Entity>> = self.string_keyed.values().map(|v| v.clone()).collect();
        values.push(self.rest.clone());
        Rc::new(Entity::Union(values).simplify())
      }
      Entity::Symbol(key) => {
        self.symbol_keyed.get(&key.id).map_or_else(|| self.rest.clone(), Rc::clone)
      }
      Entity::UnknownSymbol => {
        // TODO:
        Rc::new(Entity::Unknown)
      }
      Entity::Union(keys) => Rc::new(Entity::Union(
        keys.iter().map(|key| self.get_property(key)).collect::<Vec<Rc<Entity>>>(),
      )),
      _ => unreachable!(),
    }
  }
}

pub const UNKNOWN_OBJECT: LazyCell<ObjectEntity> = LazyCell::new(|| ObjectEntity::default());
