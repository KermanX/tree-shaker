use crate::{entity::Entity, Analyzer};
use oxc::ast::ast::{
  BigIntLiteral, BooleanLiteral, NullLiteral, NumberBase, NumericLiteral, RegExpLiteral,
  StringLiteral,
};

impl<'a> Analyzer<'a> {
  pub fn exec_string_literal(&mut self, node: &'a StringLiteral) -> Entity<'a> {
    self.factory.new_string(node.value.as_str())
  }

  pub fn exc_numeric_literal(&mut self, node: &'a NumericLiteral) -> Entity<'a> {
    if node.base == NumberBase::Float {
      self.factory.unknown_number
    } else {
      self.factory.new_number(node.value, self.allocator.alloc(node.value.to_string()))
    }
  }

  pub fn exc_big_int_literal(&mut self, node: &'a BigIntLiteral) -> Entity<'a> {
    self.factory.new_big_int(&node.raw.as_str()[..node.raw.len() - 1])
  }

  pub fn exec_boolean_literal(&mut self, node: &'a BooleanLiteral) -> Entity<'a> {
    self.factory.new_boolean(node.value)
  }

  pub fn exec_null_literal(&mut self, _node: &'a NullLiteral) -> Entity<'a> {
    self.factory.null
  }

  pub fn exec_regexp_literal(&mut self, _node: &'a RegExpLiteral<'a>) -> Entity<'a> {
    self.factory.unknown_regexp
  }
}
