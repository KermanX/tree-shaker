pub mod object;
pub mod symbol;

use std::{ops::Deref, rc::Rc, result};

use object::ObjectEntity;
use symbol::SymbolEntity;

#[derive(Debug, Clone)]
pub enum Entity {
  StringLiteral(String),
  UnknownString,

  NumberLiteral(f64),
  NonZeroNumber,
  UnknownNumber,

  BigIntLiteral(i64),
  NonZeroBigInt,
  UnknownBigInt,

  BooleanLiteral(bool),
  UnknownBoolean,

  Null,

  Undefined,

  Symbol(SymbolEntity),
  UnknownSymbol,

  Object(ObjectEntity),

  Array(),

  Union(Vec<Rc<Entity>>),

  Unknown,
}

impl Default for Entity {
  fn default() -> Self {
    Entity::Unknown
  }
}

impl Entity {
  pub fn simplified(&self) -> Entity {
    let result = match self {
      Entity::Union(values) => {
        if values.len() == 1 {
          Some(values[0].deref().clone())
        } else if values.iter().any(|value| matches!(value.deref(), Entity::Unknown)) {
          Some(Entity::Unknown)
        } else {
          None
        }
      }
      _ => todo!(),
    };
    result.map_or_else(|| self.clone(), |result| result.simplified())
  }

  pub fn get_property(&self, key: &Entity) -> Rc<Entity> {
    match self {
      Entity::Object(obj) => obj.get_property(key),
      Entity::Union(keys) => Rc::new(Entity::Union(
        keys.iter().map(|key| key.get_property(key)).collect::<Vec<Rc<Entity>>>(),
      )),
      Entity::Unknown => Rc::new(Entity::Unknown),
      _ => unreachable!(),
    }
  }

  pub fn is_null_or_undefined(&self) -> bool {
    matches!(self, Entity::Null | Entity::Undefined)
  }
}
