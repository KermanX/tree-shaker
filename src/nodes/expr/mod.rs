mod call_expression;
mod literals;
mod logical_expression;
mod object_expression;

use crate::entity::entity::Entity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::Expression;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_expression(&mut self, node: &'a Expression<'a>) -> Entity<'a> {
    let val = match node {
      Expression::NumericLiteral(node) => self.exc_numeric_literal(node),
      Expression::StringLiteral(node) => self.exec_string_literal(node),
      Expression::BooleanLiteral(node) => self.exec_boolean_literal(node),
      Expression::Identifier(node) => self.exec_identifier_reference_read(node),
      Expression::LogicalExpression(node) => self.exec_logical_expression(node),
      Expression::CallExpression(node) => self.exec_call_expression(node),
      Expression::ObjectExpression(node) => self.exec_object_expression(node),
      _ => todo!(),
    };

    val
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_expression(
    &self,
    node: Expression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    match node {
      Expression::NumericLiteral(_)
      | Expression::StringLiteral(_)
      | Expression::BooleanLiteral(_) => {
        if need_val {
          Some(node)
        } else {
          None
        }
      }

      Expression::Identifier(node) => self
        .transform_identifier_reference_read(node.unbox(), need_val)
        .map(|id| self.ast_builder.expression_from_identifier_reference(id)),
      Expression::LogicalExpression(node) => {
        self.transform_logical_expression(node.unbox(), need_val)
      }

      Expression::CallExpression(node) => self.transform_call_expression(node.unbox(), need_val),

      Expression::ObjectExpression(node) => self.transform_object_expression(node.unbox()),

      _ => todo!(),
    }
  }
}
