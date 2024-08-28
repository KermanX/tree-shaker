use crate::{
  entity::{entity::Entity, literal::LiteralEntity},
  Analyzer,
};
use oxc::ast::ast::{BigIntLiteral, BooleanLiteral, NullLiteral, NumericLiteral, StringLiteral};
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_string_literal(&mut self, node: &'a StringLiteral) -> Entity<'a> {
    Rc::new(LiteralEntity::String(node.value.as_str()))
  }

  pub(crate) fn exc_numeric_literal(&mut self, node: &'a NumericLiteral) -> Entity<'a> {
    Rc::new(LiteralEntity::Number(node.value.into(), node.raw))
  }

  pub(crate) fn exc_big_int_literal(&mut self, node: &'a BigIntLiteral) -> Entity<'a> {
    Rc::new(LiteralEntity::BigInt(node.raw.as_str()))
  }

  pub(crate) fn exec_boolean_literal(&mut self, node: &'a BooleanLiteral) -> Entity<'a> {
    Rc::new(LiteralEntity::Boolean(node.value))
  }

  pub(crate) fn exec_null_literal(&mut self, _node: &'a NullLiteral) -> Entity<'a> {
    Rc::new(LiteralEntity::Null)
  }
}
