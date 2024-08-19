use std::{cell::LazyCell, rc::Rc};

use super::Entity;
use rustc_hash::FxHashMap;

#[derive(Debug, Default, Clone)]
pub struct ObjectEntity {
  string_keyed: FxHashMap<String, Rc<Entity>>,
  symbol_keyed: FxHashMap<usize, Rc<Entity>>,
  pub rest: Option<Rc<Entity>>,
}

impl ObjectEntity {
  pub fn get_property(&self, key: &Entity) -> Rc<Entity> {
    match key {
      Entity::StringLiteral(key) => {
        self.string_keyed.get(key).or(self.rest.as_ref()).cloned().unwrap_or_default()
      }
      Entity::UnknownString => {
        let mut values: Vec<Rc<Entity>> = self.string_keyed.values().map(|v| v.clone()).collect();
        self.rest.as_ref().map(|v| values.push(v.clone()));
        Rc::new(Entity::Union(values))
      }
      Entity::Symbol(key) => {
        self.symbol_keyed.get(&key.id).or(self.rest.as_ref()).cloned().unwrap_or_default()
      }
      Entity::UnknownSymbol => {
        let mut values: Vec<Rc<Entity>> = self.symbol_keyed.values().map(|v| v.clone()).collect();
        self.rest.as_ref().map(|v| values.push(v.clone()));
        Rc::new(Entity::Union(values))
      }
      Entity::Union(keys) => Rc::new(Entity::Union(
        keys.iter().map(|key| self.get_property(key)).collect::<Vec<Rc<Entity>>>(),
      )),
      _ => unreachable!(),
    }
  }
}

pub const UNKNOWN_OBJECT: LazyCell<ObjectEntity> = LazyCell::new(|| ObjectEntity::default());
