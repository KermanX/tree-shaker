mod array_expression;
mod literals;

use crate::Analyzer;
pub use array_expression::*;
pub use literals::*;
use oxc::ast::ast::Expression;

#[allow(unused_variables)]
pub trait ExpressionAnalyzer<'a>: ArrayExpressionAnalyzer<'a> + LiteralsAnalyzer<'a> {
  fn before_expression(&self, node: &'a Expression<'a>)
  where
    Self: Analyzer<'a>,
  {
  }
  fn after_expression(&self, node: &'a Expression<'a>, value: Self::Entity) -> Self::Entity
  where
    Self: Analyzer<'a>,
  {
    value
  }

  fn exec_expression(&mut self, node: &'a Expression<'a>) -> Self::Entity
  where
    Self: Analyzer<'a>,
  {
    self.before_expression(node);

    let value = match node {
      Expression::StringLiteral(node) => self.exec_string_literal(node),
      Expression::NumericLiteral(node) => self.exec_numeric_literal(node),
      Expression::BigIntLiteral(node) => self.exc_big_int_literal(node),
      Expression::BooleanLiteral(node) => self.exec_boolean_literal(node),
      Expression::NullLiteral(node) => self.exec_null_literal(node),
      Expression::RegExpLiteral(node) => self.exec_regexp_literal(node),
      Expression::ArrayExpression(node) => self.exec_array_expression(node),

      _ => todo!(),
    };

    self.after_expression(node, value)
  }
}
