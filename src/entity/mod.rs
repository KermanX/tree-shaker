pub mod array;
pub mod convertion;
pub mod function;
pub mod object;
pub mod simplify;
pub mod symbol;

use std::{ops::Deref, rc::Rc};

use array::ArrayEntity;
use function::FunctionEntity;
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

  Null,
  Undefined,

  Symbol(SymbolEntity),
  UnknownSymbol,

  Object(ObjectEntity),

  Array(ArrayEntity),

  Function(FunctionEntity),

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

  pub fn call(&self, this: Option<&Entity>, args: &[Entity]) -> Entity {
    match self {
      Entity::Function(func) => func.call(this, args),
      Entity::Union(funcs) => {
        let mut results = vec![];
        for func in funcs {
          results.push(Rc::new(func.call(this, args)));
        }
        Entity::Union(results).simplify()
      }
      _  => Entity::Unknown,
    }
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
