use crate::TreeShaker;
use ecma_analyzer::{Analyzer, LiteralsAnalyzer};
use oxc::ast::ast::{
  BigIntLiteral, BooleanLiteral, NullLiteral, NumberBase, NumericLiteral, RegExpLiteral,
  StringLiteral,
};

impl<'a> LiteralsAnalyzer<'a> for TreeShaker<'a> {
  fn exec_string_literal(
    &mut self,
    node: &'a StringLiteral,
  ) -> <TreeShaker<'a> as Analyzer<'a>>::Entity {
    "self.factory.string(node.value.as_str())"
  }

  fn exec_numeric_literal(
    &mut self,
    node: &'a NumericLiteral,
  ) -> <TreeShaker<'a> as Analyzer<'a>>::Entity {
    ""
  }

  fn exc_big_int_literal(
    &mut self,
    node: &'a BigIntLiteral,
  ) -> <TreeShaker<'a> as Analyzer<'a>>::Entity {
    ""
  }

  fn exec_boolean_literal(
    &mut self,
    node: &'a BooleanLiteral,
  ) -> <TreeShaker<'a> as Analyzer<'a>>::Entity {
    ""
  }

  fn exec_null_literal(
    &mut self,
    _node: &'a NullLiteral,
  ) -> <TreeShaker<'a> as Analyzer<'a>>::Entity {
    ""
  }

  fn exec_regexp_literal(
    &mut self,
    _node: &'a RegExpLiteral<'a>,
  ) -> <TreeShaker<'a> as Analyzer<'a>>::Entity {
    ""
  }
}
