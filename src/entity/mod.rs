pub mod array;
pub mod convertion;
pub mod function;
pub mod object;
pub mod operations;
pub mod simplify;
pub mod symbol;

use array::ArrayEntity;
use function::FunctionEntity;
use object::ObjectEntity;
use std::rc::Rc;
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

  Null,
  Undefined,

  Symbol(SymbolEntity),
  UnknownSymbol,

  Function(FunctionEntity),
  UnknownFunction,

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
  pub fn new_unknown_boolean() -> Self {
    Entity::Union(vec![
      Rc::new(Entity::StringLiteral("true".to_string())),
      Rc::new(Entity::StringLiteral("false".to_string())),
    ])
  }

  pub fn is_null_or_undefined(&self) -> bool {
    matches!(self, Entity::Null | Entity::Undefined)
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
