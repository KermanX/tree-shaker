use super::{
  entity::{Entity, EntityTrait},
  typeof_result::TypeofResult,
  unknown::UnknownEntity,
};
use crate::{analyzer::Analyzer, utils::F64WithEq};
use oxc::{
  ast::{
    ast::{BigintBase, Expression, NumberBase, UnaryOperator},
    AstBuilder,
  },
  span::Span,
};
use rustc_hash::FxHashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LiteralEntity<'a> {
  String(&'a str),
  Number(F64WithEq, &'a str),
  BigInt(&'a str),
  Boolean(bool),
  Symbol(usize),
  Infinity(bool),
  NaN,
  Null,
  Undefined,
}

impl<'a> EntityTrait<'a> for LiteralEntity<'a> {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}
  fn consume_as_unknown(&self, _analyzer: &mut Analyzer<'a>) {}

  fn get_property(&self, analyzer: &mut Analyzer<'a>, _key: &Entity<'a>) -> (bool, Entity<'a>) {
    todo!("built-ins")
  }

  fn set_property(
    &self,
    _analyzer: &mut Analyzer<'a>,
    _key: &Entity<'a>,
    _value: Entity<'a>,
  ) -> bool {
    if matches!(self, LiteralEntity::Null | LiteralEntity::Undefined) {
      // TODO: throw warning
    }
    // No effect
    false
  }

  fn enumerate_properties(
    &self,
    _analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
    // No effect
    (false, vec![])
  }

  fn delete_property(&self, _analyzer: &mut Analyzer<'a>, _key: &Entity<'a>) -> bool {
    // No effect
    false
  }

  fn call(
    &self,
    _analyzer: &mut Analyzer<'a>,
    _this: &Entity<'a>,
    _args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    // TODO: throw warning
    (false, UnknownEntity::new_unknown())
  }

  fn r#await(&self, rc: &Entity<'a>, _analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    (false, rc.clone())
  }

  fn iterate(&self, _rc: &Entity<'a>, _analyzer: &mut Analyzer<'a>) -> (bool, Option<Entity<'a>>) {
    // TODO: throw warning
    (true, Some(UnknownEntity::new_unknown()))
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string(self.test_typeof().to_string().unwrap())
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    LiteralEntity::new_string(match self {
      LiteralEntity::String(value) => *value,
      LiteralEntity::Number(_, raw) => *raw,
      LiteralEntity::BigInt(value) => *value,
      LiteralEntity::Boolean(value) => {
        if *value {
          "true"
        } else {
          "false"
        }
      }
      LiteralEntity::Symbol(value) => todo!(),
      LiteralEntity::Infinity(positive) => {
        if *positive {
          "Infinity"
        } else {
          "-Infinity"
        }
      }
      LiteralEntity::NaN => "NaN",
      LiteralEntity::Null => "null",
      LiteralEntity::Undefined => "undefined",
    })
  }

  fn get_to_property_key(&self, rc: &Entity<'a>) -> Entity<'a> {
    match self {
      LiteralEntity::Symbol(_) => Entity::new(*self),
      _ => self.get_to_string(rc),
    }
  }

  fn get_to_array(&self, _rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    UnknownEntity::new_unknown_to_array_result(length, vec![])
  }

  fn get_to_literals(&self) -> Option<FxHashSet<LiteralEntity<'a>>> {
    let mut result = FxHashSet::default();
    result.insert(*self);
    Some(result)
  }

  fn test_typeof(&self) -> TypeofResult {
    match self {
      LiteralEntity::String(_) => TypeofResult::String,
      LiteralEntity::Number(_, _) => TypeofResult::Number,
      LiteralEntity::BigInt(_) => TypeofResult::BigInt,
      LiteralEntity::Boolean(_) => TypeofResult::Boolean,
      LiteralEntity::Symbol(_) => TypeofResult::Symbol,
      LiteralEntity::Infinity(_) => TypeofResult::Number,
      LiteralEntity::NaN => TypeofResult::Number,
      LiteralEntity::Null => TypeofResult::Object,
      LiteralEntity::Undefined => TypeofResult::Undefined,
    }
  }

  fn test_truthy(&self) -> Option<bool> {
    Some(match self {
      LiteralEntity::String(value) => !value.is_empty(),
      LiteralEntity::Number(value, _) => *value != 0.0.into(),
      LiteralEntity::BigInt(value) => !value.is_empty(),
      LiteralEntity::Boolean(value) => *value,
      LiteralEntity::Symbol(_) => true,
      LiteralEntity::Infinity(_) => true,
      LiteralEntity::NaN | LiteralEntity::Null | LiteralEntity::Undefined => false,
    })
  }

  fn test_nullish(&self) -> Option<bool> {
    Some(matches!(self, LiteralEntity::Null | LiteralEntity::Undefined))
  }
}

impl<'a> Hash for LiteralEntity<'a> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      LiteralEntity::String(value) => {
        state.write_u8(0);
        value.hash(state);
      }
      LiteralEntity::Number(_, raw) => {
        state.write_u8(1);
        raw.hash(state);
      }
      LiteralEntity::BigInt(value) => {
        state.write_u8(2);
        value.hash(state);
      }
      LiteralEntity::Boolean(value) => {
        state.write_u8(3);
        value.hash(state);
      }
      LiteralEntity::Symbol(value) => {
        state.write_u8(4);
        value.hash(state);
      }
      LiteralEntity::Infinity(positive) => {
        state.write_u8(5);
        positive.hash(state);
      }
      LiteralEntity::NaN => {
        state.write_u8(6);
      }
      LiteralEntity::Null => {
        state.write_u8(7);
      }
      LiteralEntity::Undefined => {
        state.write_u8(8);
      }
    }
  }
}

impl<'a> LiteralEntity<'a> {
  pub fn new_string(value: &'a str) -> Entity<'a> {
    Entity::new(LiteralEntity::String(value))
  }

  pub fn new_number(value: F64WithEq, raw: &'a str) -> Entity<'a> {
    Entity::new(LiteralEntity::Number(value, raw))
  }

  pub fn new_big_int(value: &'a str) -> Entity<'a> {
    Entity::new(LiteralEntity::BigInt(value))
  }

  pub fn new_boolean(value: bool) -> Entity<'a> {
    Entity::new(LiteralEntity::Boolean(value))
  }

  pub fn new_infinity(positive: bool) -> Entity<'a> {
    Entity::new(LiteralEntity::Infinity(positive))
  }

  pub fn new_nan() -> Entity<'a> {
    Entity::new(LiteralEntity::NaN)
  }

  pub fn new_null() -> Entity<'a> {
    Entity::new(LiteralEntity::Null)
  }

  pub fn new_undefined() -> Entity<'a> {
    Entity::new(LiteralEntity::Undefined)
  }

  pub fn build_expr(&self, ast_builder: &AstBuilder<'a>, span: Span) -> Expression<'a> {
    match self {
      LiteralEntity::String(value) => ast_builder.expression_string_literal(span, *value),
      LiteralEntity::Number(value, raw) => {
        ast_builder.expression_numeric_literal(span, value.0, *raw, NumberBase::Decimal)
      }
      LiteralEntity::BigInt(value) => {
        ast_builder.expression_big_int_literal(span, *value, BigintBase::Decimal)
      }
      LiteralEntity::Boolean(value) => ast_builder.expression_boolean_literal(span, *value),
      LiteralEntity::Symbol(value) => todo!(),
      LiteralEntity::Infinity(positive) => {
        if *positive {
          ast_builder.expression_identifier_reference(span, "Infinity")
        } else {
          ast_builder.expression_unary(
            span,
            UnaryOperator::UnaryNegation,
            ast_builder.expression_identifier_reference(span, "Infinity"),
          )
        }
      }
      LiteralEntity::NaN => ast_builder.expression_identifier_reference(span, "NaN"),
      LiteralEntity::Null => ast_builder.expression_null_literal(span),
      LiteralEntity::Undefined => ast_builder.expression_identifier_reference(span, "undefined"),
    }
  }

  pub fn to_string(&self) -> &'a str {
    match self {
      LiteralEntity::String(value) => *value,
      LiteralEntity::Number(_, raw) => *raw,
      LiteralEntity::BigInt(value) => *value,
      LiteralEntity::Boolean(value) => {
        if *value {
          "true"
        } else {
          "false"
        }
      }
      LiteralEntity::Symbol(value) => todo!(),
      LiteralEntity::Infinity(positive) => {
        if *positive {
          "Infinity"
        } else {
          "-Infinity"
        }
      }
      LiteralEntity::NaN => "NaN",
      LiteralEntity::Null => "null",
      LiteralEntity::Undefined => "undefined",
    }
  }
}
