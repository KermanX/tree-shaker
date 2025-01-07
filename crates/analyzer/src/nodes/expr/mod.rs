mod array_expression;
// mod arrow_function_expression;
// mod assignment_expression;
// mod await_expression;
// mod binary_expression;
// mod call_expression;
// mod chain_expression;
// mod conditional_expression;
// mod import_expression;
// mod literals;
// mod logical_expression;
// mod member_expression;
// mod meta_property;
// mod new_expression;
// mod object_expression;
// mod parenthesized_expression;
// mod private_in_expression;
// mod sequence_expression;
// mod super_expression;
// mod tagged_template_expression;
// mod template_literal;
// mod this_expression;
// mod unary_expression;
// mod update_expression;
// mod yield_expression;

pub use array_expression::*;

use crate::analyzer::Analyzer;
use oxc::ast::{ast::Expression, match_member_expression};

#[allow(unused_variables)]
pub trait ExpressionAnalyzer<'a>: ArrayExpressionAnalyzer<'a> {
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
      // match_member_expression!(Expression) => {
      //   self.exec_member_expression_read(node.to_member_expression(), false).0
      // }
      // Expression::StringLiteral(node) => self.exec_string_literal(node),
      // Expression::NumericLiteral(node) => self.exec_numeric_literal(node),
      // Expression::BigIntLiteral(node) => self.exc_big_int_literal(node),
      // Expression::BooleanLiteral(node) => self.exec_boolean_literal(node),
      // Expression::NullLiteral(node) => self.exec_null_literal(node),
      // Expression::RegExpLiteral(node) => self.exec_regexp_literal(node),
      // Expression::TemplateLiteral(node) => self.exec_template_literal(node),
      // Expression::Identifier(node) => self.exec_identifier_reference_read(node),
      // Expression::FunctionExpression(node) => self.exec_function(node),
      // Expression::ArrowFunctionExpression(node) => self.exec_arrow_function_expression(node),
      // Expression::UnaryExpression(node) => self.exec_unary_expression(node),
      // Expression::UpdateExpression(node) => self.exec_update_expression(node),
      // Expression::BinaryExpression(node) => self.exec_binary_expression(node),
      // Expression::LogicalExpression(node) => self.exec_logical_expression(node),
      // Expression::ConditionalExpression(node) => self.exec_conditional_expression(node),
      // Expression::CallExpression(node) => self.exec_call_expression(node),
      // Expression::TaggedTemplateExpression(node) => self.exec_tagged_template_expression(node),
      // Expression::AwaitExpression(node) => self.exec_await_expression(node),
      // Expression::YieldExpression(node) => self.exec_yield_expression(node),
      // Expression::ObjectExpression(node) => self.exec_object_expression(node),
      Expression::ArrayExpression(node) => self.exec_array_expression(node),
      // Expression::ParenthesizedExpression(node) => self.exec_parenthesized_expression(node),
      // Expression::SequenceExpression(node) => self.exec_sequence_expression(node),
      // Expression::AssignmentExpression(node) => self.exec_assignment_expression(node),
      // Expression::ChainExpression(node) => self.exec_chain_expression(node),
      // Expression::ImportExpression(node) => self.exec_import_expression(node),
      // Expression::MetaProperty(node) => self.exec_meta_property(node),
      // Expression::NewExpression(node) => self.exec_new_expression(node),
      // Expression::ClassExpression(node) => self.exec_class(node),
      // Expression::ThisExpression(node) => self.exec_this_expression(node),
      // Expression::Super(node) => self.exec_super(node),
      // Expression::PrivateInExpression(node) => self.exec_private_in_expression(node),

      // Expression::JSXElement(node) => self.exec_jsx_element(node),
      // Expression::JSXFragment(node) => self.exec_jsx_fragment(node),
      Expression::TSAsExpression(_)
      | Expression::TSInstantiationExpression(_)
      | Expression::TSTypeAssertion(_)
      | Expression::TSNonNullExpression(_)
      | Expression::TSSatisfiesExpression(_) => unreachable!(),

      _ => todo!(),
    };

    self.after_expression(node, value)
  }
}
