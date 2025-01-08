use crate::{analyzer::Analyzer, entity::Entity, transformer::Transformer};
use ecma_analyzer::LiteralsAnalyzer;
use oxc::ast::ast::{
  BigIntLiteral, BooleanLiteral, Expression, NullLiteral, NumberBase, NumericLiteral,
  RegExpLiteral, StringLiteral,
};

impl<'a> LiteralsAnalyzer<'a> for Analyzer<'a> {
  fn exec_string_literal(&mut self, node: &'a StringLiteral) -> Entity<'a> {
    self.factory.string(node.value.as_str())
  }

  fn exec_numeric_literal(&mut self, node: &'a NumericLiteral) -> Entity<'a> {
    if node.base == NumberBase::Float {
      self.factory.unknown_number
    } else {
      self.factory.number(node.value, None)
    }
  }

  fn exc_big_int_literal(&mut self, node: &'a BigIntLiteral) -> Entity<'a> {
    self.factory.big_int(&node.raw.as_str()[..node.raw.len() - 1])
  }

  fn exec_boolean_literal(&mut self, node: &'a BooleanLiteral) -> Entity<'a> {
    self.factory.boolean(node.value)
  }

  fn exec_null_literal(&mut self, _node: &'a NullLiteral) -> Entity<'a> {
    self.factory.null
  }

  fn exec_regexp_literal(&mut self, _node: &'a RegExpLiteral<'a>) -> Entity<'a> {
    self.factory.immutable_unknown
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_literals(
    &self,
    node: &'a Expression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    need_val.then(|| self.clone_node(node))
  }
}
