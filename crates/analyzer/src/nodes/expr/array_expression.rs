use crate::EcmaAnalyzer;
use oxc::ast::ast::{ArrayExpression, ArrayExpressionElement, SpreadElement};

pub trait ArrayExpressionAnalyzer<'a> {
  type Context;

  fn before_array_expression(&mut self, node: &'a ArrayExpression<'a>) -> Self::Context
  where
    Self: EcmaAnalyzer<'a>;

  fn init_element(
    &mut self,
    node: &'a ArrayExpressionElement<'a>,
    context: &mut Self::Context,
    value: Self::Entity,
  ) where
    Self: EcmaAnalyzer<'a>;

  fn init_spread(
    &mut self,
    node: &'a SpreadElement<'a>,
    context: &mut Self::Context,
    value: Self::Entity,
  ) where
    Self: EcmaAnalyzer<'a>;

  fn after_array_expression(&mut self, context: Self::Context) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>;

  fn exec_array_expression(&mut self, node: &'a ArrayExpression<'a>) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>,
  {
    let mut context = self.before_array_expression(node);

    for node in &node.elements {
      match node {
        ArrayExpressionElement::SpreadElement(node) => {
          let value = self.exec_expression(&node.argument);
          self.init_spread(node, &mut context, value);
        }
        ArrayExpressionElement::Elision(_node) => {
          self.init_element(node, &mut context, self.new_undefined_value());
        }
        _ => {
          let value = self.exec_expression(node.to_expression());
          self.init_element(node, &mut context, value);
        }
      }
    }

    self.after_array_expression(context)
  }
}
