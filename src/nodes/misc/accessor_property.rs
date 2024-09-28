use crate::{
  analyzer::Analyzer,
  entity::{Entity, LiteralEntity},
  transformer::Transformer,
};
use oxc::ast::{
  ast::{AccessorProperty, ClassElement},
  NONE,
};

impl<'a> Analyzer<'a> {
  pub fn exec_accessor_property(&mut self, node: &'a AccessorProperty<'a>, key: Entity<'a>) {
    let value = node
      .value
      .as_ref()
      .map_or_else(|| LiteralEntity::new_undefined(), |node| self.exec_expression(node));

    key.consume(self);
    value.consume(self);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_accessor_property(&self, node: &'a AccessorProperty<'a>) -> ClassElement<'a> {
    let AccessorProperty { r#type, span, decorators, key, value, r#static, definite, .. } = node;

    let (computed, key) = self.transform_property_key(key, true).unwrap();
    let value = value.as_ref().map(|node| self.transform_expression(node, true).unwrap());

    self.ast_builder.class_element_accessor_property(
      *r#type,
      *span,
      self.clone_node(decorators),
      key,
      value,
      computed,
      *r#static,
      *definite,
      NONE,
      None,
    )
  }
}
