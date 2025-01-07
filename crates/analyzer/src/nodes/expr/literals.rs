use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::{
  BigIntLiteral, BooleanLiteral, NullLiteral, NumericLiteral, RegExpLiteral, StringLiteral,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_string_literal(&mut self, node: &'a StringLiteral) -> H::Entity {
    self.host.new_string(node)
  }

  pub fn exec_numeric_literal(&mut self, node: &'a NumericLiteral) -> H::Entity {
    self.host.new_numeric(node)
  }

  pub fn exc_big_int_literal(&mut self, node: &'a BigIntLiteral) -> H::Entity {
    self.host.new_big_int(node)
  }

  pub fn exec_boolean_literal(&mut self, node: &'a BooleanLiteral) -> H::Entity {
    self.host.new_boolean(node)
  }

  pub fn exec_null_literal(&mut self, node: &'a NullLiteral) -> H::Entity {
    self.host.new_null(node)
  }

  pub fn exec_regexp_literal(&mut self, node: &'a RegExpLiteral<'a>) -> H::Entity {
    self.host.new_regexp(node)
  }
}
