use crate::{
  analyzer::Analyzer,
  entity::{Entity, LiteralEntity},
  transformer::Transformer,
};
use oxc::ast::{
  ast::{ClassElement, PropertyDefinition},
  NONE,
};

impl<'a> Analyzer<'a> {
  pub fn exec_property_definition(&mut self, node: &'a PropertyDefinition<'a>, key: Entity<'a>) {
    let value = node
      .value
      .as_ref()
      .map_or_else(|| LiteralEntity::new_undefined(), |node| self.exec_expression(node));

    key.consume(self);
    value.consume(self);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_property_definition(
    &self,
    node: &'a PropertyDefinition<'a>,
  ) -> ClassElement<'a> {
    let PropertyDefinition { r#type, span, decorators, key, value, r#static, .. } = node;

    let (computed, key) = self.transform_property_key(key, true).unwrap();
    let value = value.as_ref().map(|node| self.transform_expression(node, true).unwrap());

    self.ast_builder.class_element_property_definition(
      *r#type,
      *span,
      self.clone_node(decorators),
      key,
      value,
      computed,
      *r#static,
      false,
      false,
      false,
      false,
      false,
      NONE,
      None,
    )
  }
}
