use crate::{host::Host, analyzer::Analyzer};
use oxc::{
  ast::ast::{Expression, JSXAttributeValue},
  span::Span,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_attribute_value(
    &mut self,
    node: &'a Option<JSXAttributeValue<'a>>,
  ) -> H::Entity {
    if let Some(node) = node {
      match node {
        JSXAttributeValue::StringLiteral(node) => self.exec_string_literal(node),
        JSXAttributeValue::ExpressionContainer(node) => {
          self.exec_jsx_expression_container_as_attribute_value(node)
        }
        JSXAttributeValue::Element(node) => self.exec_jsx_element(node),
        JSXAttributeValue::Fragment(node) => self.exec_jsx_fragment(node),
      }
    } else {
      self.factory.r#true
    }
  }
}
