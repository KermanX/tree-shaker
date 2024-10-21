use crate::transformer::Transformer;
use oxc::ast::{
  ast::{ClassElement, PropertyDefinition},
  NONE,
};

impl<'a> Transformer<'a> {
  pub fn transform_property_definition(
    &self,
    node: &'a PropertyDefinition<'a>,
  ) -> Option<ClassElement<'a>> {
    let PropertyDefinition { r#type, span, decorators, key, value, computed, r#static, .. } = node;

    let key = self.transform_property_key(key, true).unwrap();
    let value = value.as_ref().map(|node| self.transform_expression(node, true).unwrap());

    Some(self.ast_builder.class_element_property_definition(
      *r#type,
      *span,
      self.clone_node(decorators),
      key,
      value,
      *computed,
      *r#static,
      false,
      false,
      false,
      false,
      false,
      NONE,
      None,
    ))
  }
}
