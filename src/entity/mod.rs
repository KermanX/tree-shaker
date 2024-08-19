pub mod array;
pub mod object;
pub mod symbol;

use std::{ops::Deref, rc::Rc};

use array::ArrayEntity;
use object::ObjectEntity;
use symbol::SymbolEntity;

#[derive(Debug, Clone)]
pub enum Entity {
  StringLiteral(String),
  /// `true` for numeric string, `false` for unknown
  NonEmptyString(bool),
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

  Array(ArrayEntity),

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

  pub fn to_property_key(&self) -> Entity {
    match self {
      Entity::StringLiteral(str) => Entity::StringLiteral(str.clone()),
      Entity::NonEmptyString(numeric) => Entity::NonEmptyString(numeric.clone()),
      Entity::NumberLiteral(num) => Entity::StringLiteral(num.to_string()),
      Entity::BigIntLiteral(num) => Entity::StringLiteral(num.to_string()),
      Entity::BooleanLiteral(bool) => Entity::StringLiteral(bool.to_string()),
      Entity::UnknownBoolean => Entity::Union(vec![
        Rc::new(Entity::StringLiteral("true".to_string())),
        Rc::new(Entity::StringLiteral("false".to_string())),
      ]),
      Entity::Null => Entity::StringLiteral("null".to_string()),
      Entity::Undefined => Entity::StringLiteral("undefined".to_string()),
      Entity::Symbol(symbol) => Entity::Symbol(symbol.clone()),
      Entity::UnknownSymbol => Entity::UnknownSymbol,

      Entity::NonZeroNumber
      | Entity::UnknownNumber
      | Entity::NonZeroBigInt
      | Entity::UnknownBigInt => Entity::NonEmptyString(true),

      // TODO: Side effect in toString
      Entity::Object(_) | Entity::Array(_) => Entity::UnknownString,

      Entity::UnknownString | Entity::Unknown => Entity::UnknownString,

      Entity::Union(values) => {
        Entity::Union(values.iter().map(|value| Rc::new(value.to_string())).collect()).simplified()
      }
    }
  }

  pub fn to_string(&self) -> Entity {
    match self.to_property_key() {
      Entity::Symbol(_) | Entity::UnknownSymbol => Entity::NonEmptyString(false),
      str => str,
    }
  }

  pub fn is_numeric(&self) -> bool {
    matches!(
      self,
      Entity::NumberLiteral(_)
        | Entity::NonZeroNumber
        | Entity::UnknownNumber
        | Entity::BigIntLiteral(_)
        | Entity::NonZeroBigInt
        | Entity::UnknownBigInt
        | Entity::NonEmptyString(true)
    )
  }
}
