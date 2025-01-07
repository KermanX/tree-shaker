use super::entity::EntityHost;
use oxc::ast::ast::*;

pub trait PrimitivesHost<'a>: EntityHost<'a> {
  fn new_undefined(&self) -> Self::Entity;
  fn new_numeric(&self, node: &'a NumericLiteral<'a>) -> Self::Entity;
  fn new_big_int(&self, node: &'a BigIntLiteral<'a>) -> Self::Entity;
  fn new_boolean(&self, node: &'a BooleanLiteral) -> Self::Entity;
  fn new_null(&self, node: &'a NullLiteral) -> Self::Entity;
  fn new_regexp(&self, node: &'a RegExpLiteral<'a>) -> Self::Entity;
  fn new_string(&self, node: &'a StringLiteral<'a>) -> Self::Entity;
}
