mod arrow_function_expression;
mod assignment_expression;
mod await_expression;
mod binary_expression;
mod call_expression;
mod chain_expression;
mod conditional_expression;
mod import_expression;
mod literals;
mod this_expression;
mod logical_expression;
mod member_expression;
mod meta_property;
mod new_expression;
mod object_expression;
mod parenthesized_expression;
mod sequence_expression;
mod tagged_template_expression;
mod template_literal;
mod unary_expression;

use crate::ast::AstType2;
use crate::build_effect;
use crate::entity::collector::LiteralCollector;
use crate::entity::entity::Entity;
use crate::entity::literal::LiteralEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::allocator::CloneIn;
use oxc::ast::ast::Expression;
use oxc::ast::match_member_expression;
use oxc::span::{GetSpan, Span};

const AST_TYPE: AstType2 = AstType2::Expression;

#[derive(Debug, Default)]
struct Data<'a> {
  collector: LiteralCollector<'a>,
}

impl<'a> Analyzer<'a> {
  pub fn exec_expression(&mut self, node: &'a Expression<'a>) -> Entity<'a> {
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
      Expression::TaggedTemplateExpression(node) => self.exec_tagged_template_expression(node),
      Expression::AwaitExpression(node) => self.exec_await_expression(node),
      Expression::ObjectExpression(node) => self.exec_object_expression(node),
      Expression::ParenthesizedExpression(node) => self.exec_parenthesized_expression(node),
      Expression::SequenceExpression(node) => self.exec_sequence_expression(node),
      Expression::AssignmentExpression(node) => self.exec_assignment_expression(node),
      Expression::ChainExpression(node) => self.exec_chain_expression(node),
      Expression::ImportExpression(node) => self.exec_import_expression(node),
      Expression::MetaProperty(node) => self.exec_meta_property(node),
      Expression::NewExpression(node) => self.exec_new_expression(node),
      Expression::ClassExpression(node) => self.exec_class(node, false),
      Expression::ThisExpression(node) => self.exec_this_expression(node),
      _ => todo!("Expr at span {:?}", node.span()),
    };

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.collector.collect(entity)
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_expression(
    &self,
    node: &'a Expression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let span = node.span();
    let literal = need_val.then(|| data.collector.build_expr(&self.ast_builder, span)).flatten();
    let need_val = need_val && literal.is_none();

    let inner = match node {
      match_member_expression!(Expression) => {
        self.transform_member_expression_read(node.to_member_expression(), need_val)
      }
      Expression::StringLiteral(_)
      | Expression::NumericLiteral(_)
      | Expression::BigIntLiteral(_)
      | Expression::BooleanLiteral(_)
      | Expression::NullLiteral(_)
      | Expression::RegExpLiteral(_) => {
        if need_val {
          Some(node.clone_in(&self.allocator))
        } else {
          None
        }
      }
      Expression::TemplateLiteral(node) => self.transform_template_literal(node, need_val),
      Expression::Identifier(node) => self
        .transform_identifier_reference_read(node, need_val)
        .map(|id| self.ast_builder.expression_from_identifier_reference(id)),
      Expression::FunctionExpression(node) => self
        .transform_function(node, need_val)
        .map(|f| self.ast_builder.expression_from_function(f)),
      Expression::ArrowFunctionExpression(node) => {
        self.transform_arrow_function_expression(node, need_val)
      }
      Expression::UnaryExpression(node) => self.transform_unary_expression(node, need_val),
      Expression::BinaryExpression(node) => self.transform_binary_expression(node, need_val),
      Expression::LogicalExpression(node) => self.transform_logical_expression(node, need_val),
      Expression::ConditionalExpression(node) => {
        self.transform_conditional_expression(node, need_val)
      }
      Expression::CallExpression(node) => self.transform_call_expression(node, need_val),
      Expression::TaggedTemplateExpression(node) => {
        self.transform_tagged_template_expression(node, need_val)
      }
      Expression::AwaitExpression(node) => self.transform_await_expression(node, need_val),
      Expression::ObjectExpression(node) => self.transform_object_expression(node, need_val),
      Expression::ParenthesizedExpression(node) => {
        self.transform_parenthesized_expression(node, need_val)
      }
      Expression::SequenceExpression(node) => self.transform_sequence_expression(node, need_val),
      Expression::AssignmentExpression(node) => {
        self.transform_assignment_expression(node, need_val)
      }
      Expression::ChainExpression(node) => self.transform_chain_expression(node, need_val),
      Expression::ImportExpression(node) => self.transform_import_expression(node, need_val),
      Expression::MetaProperty(node) => self.transform_meta_property(node, need_val),
      Expression::NewExpression(node) => self.transform_new_expression(node, need_val),
      Expression::ClassExpression(node) => self
        .transform_class(node, need_val)
        .map(|class| self.ast_builder.expression_from_class(class)),
      Expression::ThisExpression(node) => self.transform_this_expression(node, need_val),
      _ => todo!(),
    };

    if let Some(literal) = literal {
      Some(build_effect!(&self.ast_builder, span, inner; literal))
    } else {
      inner
    }
  }

  // This is not good
  pub fn get_expression_collected_literal(&self, span: Span) -> Option<LiteralEntity<'a>> {
    let data = self.get_data_by_span::<Data>(AST_TYPE, span);
    data.collector.collected()
  }
}
