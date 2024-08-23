use super::Entity;
use crate::{transformer::Transformer, utils::F64WithEq};
use oxc::{
  ast::ast::{BigintBase, Expression, NumberBase},
  span::Span,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) enum SimpleLiteral {
  #[default]
  None,

  String(String),
  Number(F64WithEq),
  BigInt(i64),
  Boolean(bool),
  Null,
  Undefined,
}

impl Entity {
  pub(crate) fn to_simple_literal(&self) -> SimpleLiteral {
    match self.simplify() {
      Entity::StringLiteral(str) => SimpleLiteral::String(str),
      Entity::NumberLiteral(num) => SimpleLiteral::Number(num.into()),
      Entity::BigIntLiteral(num) => SimpleLiteral::BigInt(num),
      Entity::BooleanLiteral(bool) => SimpleLiteral::Boolean(bool),
      Entity::Null => SimpleLiteral::Null,
      Entity::Undefined => SimpleLiteral::Undefined,
      _ => SimpleLiteral::None,
    }
  }
}

pub(crate) fn combine_simple_literal(this: &mut SimpleLiteral, other: &Entity) {
  let other = other.to_simple_literal();
  if matches!(this, SimpleLiteral::None) {
    *this = other;
  } else if !matches!(other, SimpleLiteral::None) && this != &other {
    *this = SimpleLiteral::None;
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn build_simple_literal(
    &self,
    span: Span,
    value: &SimpleLiteral,
  ) -> Option<Expression<'a>> {
    Some(match value {
      SimpleLiteral::None => return None,
      SimpleLiteral::String(str) => self.ast_builder.expression_string_literal(span, str.clone()),
      SimpleLiteral::Number(num) => self.ast_builder.expression_numeric_literal(
        span,
        (*num).0,
        num.0.to_string(),
        NumberBase::Decimal,
      ),
      SimpleLiteral::BigInt(num) => {
        self.ast_builder.expression_big_int_literal(span, num.to_string(), BigintBase::Decimal)
      }
      SimpleLiteral::Boolean(bool) => self.ast_builder.expression_boolean_literal(span, *bool),
      SimpleLiteral::Null => self.ast_builder.expression_null_literal(span),
      SimpleLiteral::Undefined => {
        self.ast_builder.expression_identifier_reference(span, "undefined")
      }
    })
  }
}
