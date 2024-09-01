mod arrow_function_expression;
mod assignment_expression;
mod binary_expression;
mod call_expression;
mod chain_expression;
mod conditional_expression;
mod literals;
mod logical_expression;
mod member_expression;
mod object_expression;
mod parenthesized_expression;
mod sequence_expression;
mod template_literal;
mod unary_expression;

use crate::ast::AstType2;
use crate::build_effect;
use crate::entity::collector::LiteralCollector;
use crate::entity::entity::Entity;
use crate::entity::literal::LiteralEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::Expression;
use oxc::ast::match_member_expression;
use oxc::span::{GetSpan, Span};

const AST_TYPE: AstType2 = AstType2::Expression;

#[derive(Debug, Default)]
struct Data<'a> {
  collector: LiteralCollector<'a>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_expression(&mut self, node: &'a Expression<'a>) -> Entity<'a> {
    let entity = match node {
      match_member_expression!(Expression) => {
        self.exec_member_expression_read(node.to_member_expression())
      }
      Expression::StringLiteral(node) => self.exec_string_literal(node),
      Expression::NumericLiteral(node) => self.exc_numeric_literal(node),
      Expression::BigIntLiteral(node) => self.exc_big_int_literal(node),
      Expression::BooleanLiteral(node) => self.exec_boolean_literal(node),
      Expression::NullLiteral(node) => self.exec_null_literal(node),
      Expression::RegExpLiteral(node) => self.exec_regexp_literal(node),
      Expression::TemplateLiteral(node) => self.exec_template_literal(node),
      Expression::Identifier(node) => self.exec_identifier_reference_read(node),
      Expression::FunctionExpression(node) => self.exec_function(node, false),
      Expression::ArrowFunctionExpression(node) => self.exec_arrow_function_expression(node),
      Expression::UnaryExpression(node) => self.exec_unary_expression(node),
      Expression::BinaryExpression(node) => self.exec_binary_expression(node),
      Expression::LogicalExpression(node) => self.exec_logical_expression(node),
      Expression::ConditionalExpression(node) => self.exec_conditional_expression(node),
      Expression::CallExpression(node) => self.exec_call_expression(node),
      Expression::ObjectExpression(node) => self.exec_object_expression(node),
      Expression::ParenthesizedExpression(node) => self.exec_parenthesized_expression(node),
      Expression::SequenceExpression(node) => self.exec_sequence_expression(node),
      Expression::AssignmentExpression(node) => self.exec_assignment_expression(node),
      Expression::ChainExpression(node) => self.exec_chain_expression(node),
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
      match_member_expression!(Expression) => {
        self.transform_member_expression_read(node.try_into().unwrap(), need_val)
      }
      Expression::StringLiteral(_)
      | Expression::NumericLiteral(_)
      | Expression::BigIntLiteral(_)
      | Expression::BooleanLiteral(_)
      | Expression::NullLiteral(_)
      | Expression::RegExpLiteral(_) => {
        if need_val {
          Some(node)
        } else {
          None
        }
      }
      Expression::TemplateLiteral(node) => self.transform_template_literal(node.unbox(), need_val),
      Expression::Identifier(node) => self
        .transform_identifier_reference_read(node.unbox(), need_val)
        .map(|id| self.ast_builder.expression_from_identifier_reference(id)),
      Expression::FunctionExpression(node) => self
        .transform_function(node.unbox(), need_val)
        .map(|f| self.ast_builder.expression_from_function(f)),
      Expression::ArrowFunctionExpression(node) => {
        self.transform_arrow_function_expression(node.unbox(), need_val)
      }
      Expression::UnaryExpression(node) => self.transform_unary_expression(node.unbox(), need_val),
      Expression::BinaryExpression(node) => {
        self.transform_binary_expression(node.unbox(), need_val)
      }
      Expression::LogicalExpression(node) => {
        self.transform_logical_expression(node.unbox(), need_val)
      }
      Expression::ConditionalExpression(node) => {
        self.transform_conditional_expression(node.unbox(), need_val)
      }
      Expression::CallExpression(node) => self.transform_call_expression(node.unbox(), need_val),
      Expression::ObjectExpression(node) => {
        self.transform_object_expression(node.unbox(), need_val)
      }
      Expression::ParenthesizedExpression(node) => {
        self.transform_parenthesized_expression(node.unbox(), need_val)
      }
      Expression::SequenceExpression(node) => {
        self.transform_sequence_expression(node.unbox(), need_val)
      }
      Expression::AssignmentExpression(node) => {
        self.transform_assignment_expression(node.unbox(), need_val)
      }
      Expression::ChainExpression(node) => self.transform_chain_expression(node.unbox(), need_val),
      _ => todo!(),
    };

    if let Some(literal) = literal {
      Some(build_effect!(&self.ast_builder, span, inner; literal))
    } else {
      inner
    }
  }

  // This is not good
  pub(crate) fn get_expression_collected_literal(&self, span: Span) -> Option<LiteralEntity<'a>> {
    let data = self.get_data_by_span::<Data>(AST_TYPE, span);
    data.collector.collected()
  }
}
