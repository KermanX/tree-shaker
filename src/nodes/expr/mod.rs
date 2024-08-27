mod call_expression;
mod conditional_expression;
mod literals;
mod logical_expression;
mod object_expression;
mod parenthesized_expression;
mod sequence_expression;
mod static_member_expression;

use crate::ast::AstType2;
use crate::build_effect;
use crate::entity::collector::LiteralCollector;
use crate::entity::entity::Entity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::Expression;
use oxc::span::GetSpan;

const AST_TYPE: AstType2 = AstType2::Expression;

#[derive(Debug, Default)]
struct Data<'a> {
  collector: LiteralCollector<'a>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_expression(&mut self, node: &'a Expression<'a>) -> Entity<'a> {
    let entity = match node {
      Expression::NumericLiteral(node) => self.exc_numeric_literal(node),
      Expression::StringLiteral(node) => self.exec_string_literal(node),
      Expression::BooleanLiteral(node) => self.exec_boolean_literal(node),
      Expression::Identifier(node) => self.exec_identifier_reference_read(node),
      Expression::LogicalExpression(node) => self.exec_logical_expression(node),
      Expression::ConditionalExpression(node) => self.exec_conditional_expression(node),
      Expression::CallExpression(node) => self.exec_call_expression(node),
      Expression::StaticMemberExpression(node) => self.exec_static_member_expression(node),
      Expression::ObjectExpression(node) => self.exec_object_expression(node),
      Expression::ParenthesizedExpression(node) => self.exec_parenthesized_expression(node),
      Expression::SequenceExpression(node) => self.exec_sequence_expression(node),
      _ => todo!("Expr at span {:?}", node.span()),
    };

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.collector.collect(entity)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_expression(
    &mut self,
    node: Expression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let span = node.span();
    let literal = need_val.then(|| data.collector.build_expr(&self.ast_builder, span)).flatten();
    let need_val = need_val && literal.is_none();

    let inner = match node {
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
      Expression::ConditionalExpression(node) => {
        self.transform_conditional_expression(node.unbox(), need_val)
      }
      Expression::CallExpression(node) => self.transform_call_expression(node.unbox(), need_val),
      Expression::StaticMemberExpression(node) => {
        self.transform_static_member_expression(node.unbox(), need_val)
      }
      Expression::ObjectExpression(node) => {
        self.transform_object_expression(node.unbox(), need_val)
      }
      Expression::ParenthesizedExpression(node) => {
        self.transform_parenthesized_expression(node.unbox(), need_val)
      }
      Expression::SequenceExpression(node) => {
        self.transform_sequence_expression(node.unbox(), need_val)
      }
      _ => todo!(),
    };

    if let Some(literal) = literal {
      Some(build_effect!(&self.ast_builder, span, inner; literal))
    } else {
      inner
    }
  }
}
