use crate::{analyzer::Analyzer, entity::Entity, transformer::Transformer};
use oxc::ast::ast::{Expression, JSXAttributeValue};

impl<'a> Analyzer<'a> {
  pub fn exec_jsx_attribute_value(
    &mut self,
    node: &'a Option<JSXAttributeValue<'a>>,
  ) -> Entity<'a> {
    if let Some(node) = node {
      match node {
        JSXAttributeValue::StringLiteral(node) => self.exec_string_literal(&node),
        JSXAttributeValue::ExpressionContainer(node) => {
          self.exec_jsx_expression_container_as_attribute_value(&node)
        }
        JSXAttributeValue::Element(node) => self.exec_jsx_element(&node),
        JSXAttributeValue::Fragment(node) => self.exec_jsx_fragment(&node),
      }
    } else {
      self.factory.r#true
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_jsx_attribute_value_effect_only(
    &self,
    node: &'a Option<JSXAttributeValue<'a>>,
  ) -> Option<Expression<'a>> {
    if let Some(node) = node {
      match node {
        JSXAttributeValue::StringLiteral(_node) => None,
        JSXAttributeValue::ExpressionContainer(node) => {
          self.transform_jsx_expression_container_effect_only(&node)
        }
        JSXAttributeValue::Element(node) => self.transform_jsx_element_effect_only(&node),
        JSXAttributeValue::Fragment(node) => self.transform_jsx_fragment_effect_only(&node),
      }
    } else {
      None
    }
  }

  pub fn transform_jsx_attribute_value_need_val(
    &self,
    node: &'a Option<JSXAttributeValue<'a>>,
  ) -> Option<JSXAttributeValue<'a>> {
    node.as_ref().map(|node| match node {
      JSXAttributeValue::StringLiteral(node) => {
        self.ast_builder.jsx_attribute_value_from_string_literal(self.clone_node(node))
      }
      JSXAttributeValue::ExpressionContainer(node) => {
        self.ast_builder.jsx_attribute_value_from_jsx_expression_container(
          self.transform_jsx_expression_container_need_val(&node),
        )
      }
      JSXAttributeValue::Element(node) => self
        .ast_builder
        .jsx_attribute_value_from_jsx_element(self.transform_jsx_element_need_val(&node)),
      JSXAttributeValue::Fragment(node) => self
        .ast_builder
        .jsx_attribute_value_from_jsx_fragment(self.transform_jsx_fragment_need_val(&node)),
    })
  }
}
