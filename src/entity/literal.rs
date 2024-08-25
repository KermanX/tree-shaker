use super::entity::{Entity, EntityTrait};
use crate::{analyzer::Analyzer, utils::F64WithEq};
use oxc::{
  ast::{
    ast::{BigintBase, Expression, NumberBase},
    AstBuilder,
  },
  span::Span,
};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum LiteralEntity<'a> {
  String(&'a str),
  Number(F64WithEq, &'a str),
  BigInt(&'a str),
  Boolean(bool),
  Symbol(usize),
  Null,
  Undefined,
}

impl<'a> EntityTrait<'a> for LiteralEntity<'a> {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}

  fn get_property(&self, key: &Entity<'a>) -> Entity<'a> {
    todo!()
  }

  fn get_literal(&self) -> Option<LiteralEntity<'a>> {
    Some(*self)
  }

  fn test_truthy(&self) -> Option<bool> {
    Some(match self {
      LiteralEntity::String(value) => !value.is_empty(),
      LiteralEntity::Number(value, _) => *value != 0.0.into(),
      LiteralEntity::BigInt(value) => !value.is_empty(),
      LiteralEntity::Boolean(value) => *value,
      LiteralEntity::Symbol(_) => true,
      LiteralEntity::Null | LiteralEntity::Undefined => false,
    })
  }

  fn test_nullish(&self) -> Option<bool> {
    Some(matches!(self, LiteralEntity::Null | LiteralEntity::Undefined))
  }
}

impl<'a> LiteralEntity<'a> {
  pub(crate) fn new_string(value: &'a str) -> Entity<'a> {
    Rc::new(LiteralEntity::String(value))
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
      LiteralEntity::Null => ast_builder.expression_null_literal(span),
      LiteralEntity::Undefined => ast_builder.expression_identifier_reference(span, "undefined"),
    }
  }
}
