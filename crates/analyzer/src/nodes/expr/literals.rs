use crate::Analyzer;
use oxc::ast::ast::{
  BigIntLiteral, BooleanLiteral, NullLiteral, NumericLiteral, RegExpLiteral, StringLiteral,
};

pub trait LiteralsAnalyzer<'a> {
  fn exec_string_literal(&mut self, node: &'a StringLiteral) -> Self::Entity
  where
    Self: Analyzer<'a>;

  fn exec_numeric_literal(&mut self, node: &'a NumericLiteral) -> Self::Entity
  where
    Self: Analyzer<'a>;

  fn exc_big_int_literal(&mut self, node: &'a BigIntLiteral) -> Self::Entity
  where
    Self: Analyzer<'a>;

  fn exec_boolean_literal(&mut self, node: &'a BooleanLiteral) -> Self::Entity
  where
    Self: Analyzer<'a>;

  fn exec_null_literal(&mut self, _node: &'a NullLiteral) -> Self::Entity
  where
    Self: Analyzer<'a>;

  fn exec_regexp_literal(&mut self, _node: &'a RegExpLiteral<'a>) -> Self::Entity
  where
    Self: Analyzer<'a>;
}
