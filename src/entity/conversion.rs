use super::{array::ArrayEntity, EntityValue};
use std::rc::Rc;

impl EntityValue {
  pub fn to_property_key(&self) -> EntityValue {
    match self {
      EntityValue::StringLiteral(str) => EntityValue::StringLiteral(str.clone()),
      EntityValue::NonEmptyString(numeric) => EntityValue::NonEmptyString(numeric.clone()),
      EntityValue::NumberLiteral(num) => EntityValue::StringLiteral(num.to_string()),
      EntityValue::BigIntLiteral(num) => EntityValue::StringLiteral(num.to_string()),
      EntityValue::BooleanLiteral(bool) => EntityValue::StringLiteral(bool.to_string()),
      EntityValue::Null => EntityValue::StringLiteral("null".to_string()),
      EntityValue::Undefined => EntityValue::StringLiteral("undefined".to_string()),
      EntityValue::Symbol(symbol) => EntityValue::Symbol(symbol.clone()),
      EntityValue::UnknownSymbol => EntityValue::UnknownSymbol,

      EntityValue::NonZeroNumber
      | EntityValue::UnknownNumber
      | EntityValue::NonZeroBigInt
      | EntityValue::UnknownBigInt
      | EntityValue::Function(_)
      | EntityValue::UnknownFunction => EntityValue::NonEmptyString(true),

      // TODO: Side effect in toString
      EntityValue::Object(_) | EntityValue::Array(_) => EntityValue::UnknownString,

      EntityValue::UnknownString | EntityValue::Unknown => EntityValue::UnknownString,

      EntityValue::Union(values) => {
        EntityValue::Union(values.iter().map(|value| Rc::new(value.to_string())).collect())
          .simplify()
      }
    }
  }

  pub fn to_string(&self) -> EntityValue {
    match self.to_property_key() {
      EntityValue::Symbol(_) | EntityValue::UnknownSymbol => EntityValue::NonEmptyString(false),
      str => str,
    }
  }

  /// `None` for unknown
  pub fn to_boolean(&self) -> Option<bool> {
    match self {
      EntityValue::StringLiteral(str) => Some(!str.is_empty()),
      EntityValue::NumberLiteral(num) => Some(*num != 0.0),
      EntityValue::BigIntLiteral(num) => Some(*num != 0),
      EntityValue::BooleanLiteral(bool) => Some(bool.clone()),
      EntityValue::NonEmptyString(_)
      | EntityValue::NonZeroNumber
      | EntityValue::NonZeroBigInt
      | EntityValue::Symbol(_)
      | EntityValue::UnknownSymbol
      | EntityValue::Object(_)
      | EntityValue::Array(_)
      | EntityValue::Function(_)
      | EntityValue::UnknownFunction => Some(true),
      EntityValue::Null | EntityValue::Undefined => Some(false),
      EntityValue::UnknownString
      | EntityValue::UnknownNumber
      | EntityValue::UnknownBigInt
      | EntityValue::Unknown => None,
      EntityValue::Union(values) => {
        let boolean = values[0].to_boolean()?;
        for value in &values[1..] {
          if value.to_boolean()? != boolean {
            return None;
          }
        }
        Some(boolean)
      }
    }
  }

  /// `None` for unknown
  pub fn is_null_or_undefined(&self) -> Option<bool> {
    match self {
      EntityValue::Null | EntityValue::Undefined => Some(true),
      EntityValue::Union(values) => {
        let nullable = values[0].is_null_or_undefined()?;
        for value in &values[1..] {
          if value.is_null_or_undefined()? != nullable {
            return None;
          }
        }
        Some(nullable)
      }
      EntityValue::Unknown => None,
      _ => Some(false),
    }
  }

  pub fn to_array(&self) -> ArrayEntity {
    todo!()
  }
}
