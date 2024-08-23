pub mod array;
pub mod conversion;
pub mod function;
pub mod object;
pub mod operations;
pub mod simple_literal;
pub mod simplify;
pub mod symbol;

use array::ArrayEntity;
use function::FunctionEntity;
use object::ObjectEntity;
use std::rc::Rc;
use symbol::SymbolEntity;

#[derive(Debug, Clone)]
pub enum EntityValue {
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

  Null,
  Undefined,

  Symbol(SymbolEntity),
  UnknownSymbol,

  Function(FunctionEntity),
  UnknownFunction,

  Object(ObjectEntity),

  Array(ArrayEntity),

  Union(Vec<Rc<EntityValue>>),

  Unknown,
}

impl Default for EntityValue {
  fn default() -> Self {
    EntityValue::Unknown
  }
}

impl EntityValue {
  pub fn new_unknown_boolean() -> Self {
    EntityValue::Union(vec![
      Rc::new(EntityValue::StringLiteral("true".to_string())),
      Rc::new(EntityValue::StringLiteral("false".to_string())),
    ])
  }

  pub fn is_numeric(&self) -> bool {
    matches!(
      self,
      EntityValue::NumberLiteral(_)
        | EntityValue::NonZeroNumber
        | EntityValue::UnknownNumber
        | EntityValue::BigIntLiteral(_)
        | EntityValue::NonZeroBigInt
        | EntityValue::UnknownBigInt
        | EntityValue::NonEmptyString(true)
    )
  }
}
