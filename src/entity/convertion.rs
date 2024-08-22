use super::{array::ArrayEntity, Entity};
use std::rc::Rc;

impl Entity {
  pub fn to_property_key(&self) -> Entity {
    match self {
      Entity::StringLiteral(str) => Entity::StringLiteral(str.clone()),
      Entity::NonEmptyString(numeric) => Entity::NonEmptyString(numeric.clone()),
      Entity::NumberLiteral(num) => Entity::StringLiteral(num.to_string()),
      Entity::BigIntLiteral(num) => Entity::StringLiteral(num.to_string()),
      Entity::BooleanLiteral(bool) => Entity::StringLiteral(bool.to_string()),
      Entity::Null => Entity::StringLiteral("null".to_string()),
      Entity::Undefined => Entity::StringLiteral("undefined".to_string()),
      Entity::Symbol(symbol) => Entity::Symbol(symbol.clone()),
      Entity::UnknownSymbol => Entity::UnknownSymbol,

      Entity::NonZeroNumber
      | Entity::UnknownNumber
      | Entity::NonZeroBigInt
      | Entity::UnknownBigInt
      | Entity::Function(_)
      | Entity::UnknownFunction => Entity::NonEmptyString(true),

      // TODO: Side effect in toString
      Entity::Object(_) | Entity::Array(_) => Entity::UnknownString,

      Entity::UnknownString | Entity::Unknown => Entity::UnknownString,

      Entity::Union(values) => {
        Entity::Union(values.iter().map(|value| Rc::new(value.to_string())).collect()).simplify()
      }
    }
  }

  pub fn to_string(&self) -> Entity {
    match self.to_property_key() {
      Entity::Symbol(_) | Entity::UnknownSymbol => Entity::NonEmptyString(false),
      str => str,
    }
  }

  pub fn to_boolean(&self) -> Entity {
    match self {
      Entity::StringLiteral(str) => Entity::BooleanLiteral(!str.is_empty()),
      Entity::NumberLiteral(num) => Entity::BooleanLiteral(*num != 0.0),
      Entity::BigIntLiteral(num) => Entity::BooleanLiteral(*num != 0),
      Entity::BooleanLiteral(bool) => Entity::BooleanLiteral(bool.clone()),
      Entity::NonEmptyString(_)
      | Entity::NonZeroNumber
      | Entity::NonZeroBigInt
      | Entity::Symbol(_)
      | Entity::UnknownSymbol
      | Entity::Object(_)
      | Entity::Array(_)
      | Entity::Function(_)
      | Entity::UnknownFunction => Entity::BooleanLiteral(true),
      Entity::Null | Entity::Undefined => Entity::BooleanLiteral(false),
      Entity::UnknownString | Entity::UnknownNumber | Entity::UnknownBigInt | Entity::Unknown => {
        Entity::new_unknown_boolean()
      }
      Entity::Union(values) => {
        Entity::Union(values.iter().map(|value| Rc::new(value.to_boolean())).collect())
      }
    }
  }

  /// `None` for unknown
  pub fn is_null_or_undefined(&self) -> Option<bool> {
    match self {
      Entity::Null | Entity::Undefined => Some(true),
      Entity::Union(values) => {
        let nullable = values[0].is_null_or_undefined()?;
        for value in &values[1..] {
          if value.is_null_or_undefined()? != nullable {
            return None;
          }
        }
        Some(nullable)
      }
      Entity::Unknown => None,
      _ => Some(false),
    }
  }

  pub fn to_array(&self) -> ArrayEntity {
    todo!()
  }
}
