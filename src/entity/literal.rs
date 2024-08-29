use super::{
  entity::{Entity, EntityTrait},
  typeof_result::TypeofResult,
};
use crate::{analyzer::Analyzer, utils::F64WithEq};
use oxc::{
  ast::{
    ast::{BigintBase, Expression, NumberBase},
    AstBuilder,
  },
  span::Span,
};
use rustc_hash::FxHashSet;
use std::{
  hash::{Hash, Hasher},
  rc::Rc,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum LiteralEntity<'a> {
  String(&'a str),
  Number(F64WithEq, &'a str),
  BigInt(&'a str),
  Boolean(bool),
  Symbol(usize),
  NaN,
  Null,
  Undefined,
}

impl<'a> EntityTrait<'a> for LiteralEntity<'a> {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}
  fn consume_as_unknown(&self, _analyzer: &mut Analyzer<'a>) {}

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string(self.test_typeof().to_string().unwrap())
  }

  fn get_to_string(&self) -> Entity<'a> {
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
      LiteralEntity::NaN => "NaN",
      LiteralEntity::Null => "null",
      LiteralEntity::Undefined => "undefined",
    })
  }

  fn get_to_property_key(&self) -> Entity<'a> {
    match self {
      LiteralEntity::Symbol(_) => Rc::new(self.clone()),
      _ => self.get_to_string(),
    }
  }

  fn get_property(&self, key: &Entity<'a>) -> Entity<'a> {
    todo!()
  }

  fn get_to_array(&self, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    unreachable!()
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
      LiteralEntity::NaN => {
        state.write_u8(5);
      }
      LiteralEntity::Null => {
        state.write_u8(6);
      }
      LiteralEntity::Undefined => {
        state.write_u8(7);
      }
    }
  }
}

impl<'a> LiteralEntity<'a> {
  pub(crate) fn new_string(value: &'a str) -> Entity<'a> {
    Rc::new(LiteralEntity::String(value))
  }

  pub(crate) fn new_number(value: F64WithEq, raw: &'a str) -> Entity<'a> {
    Rc::new(LiteralEntity::Number(value, raw))
  }

  pub(crate) fn new_big_int(value: &'a str) -> Entity<'a> {
    Rc::new(LiteralEntity::BigInt(value))
  }

  pub(crate) fn new_boolean(value: bool) -> Entity<'a> {
    Rc::new(LiteralEntity::Boolean(value))
  }

  pub(crate) fn new_undefined() -> Entity<'a> {
    Rc::new(LiteralEntity::Undefined)
  }

  pub(crate) fn build_expr(&self, ast_builder: &AstBuilder<'a>, span: Span) -> Expression<'a> {
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
      LiteralEntity::NaN => ast_builder.expression_identifier_reference(span, "NaN"),
      LiteralEntity::Null => ast_builder.expression_null_literal(span),
      LiteralEntity::Undefined => ast_builder.expression_identifier_reference(span, "undefined"),
    }
  }

  pub(crate) fn to_string(&self) -> String {
    match self {
      LiteralEntity::String(value) => value.to_string(),
      LiteralEntity::Number(_value, raw) => raw.to_string(),
      LiteralEntity::BigInt(value) => value.to_string(),
      LiteralEntity::Boolean(value) => value.to_string(),
      LiteralEntity::Symbol(value) => value.to_string(),
      LiteralEntity::NaN => "NaN".to_string(),
      LiteralEntity::Null => "null".to_string(),
      LiteralEntity::Undefined => "undefined".to_string(),
    }
  }
}
