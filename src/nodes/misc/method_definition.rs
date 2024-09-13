use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::{ClassElement, MethodDefinition};

impl<'a> Analyzer<'a> {
  pub fn exec_method_definition(&mut self, node: &'a MethodDefinition<'a>) {
    let key = self.exec_property_key(&node.key);
    let value = self.exec_function(&node.value);

    key.consume_self(self);
    value.consume_as_unknown(self);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_method_definition(&self, node: &'a MethodDefinition<'a>) -> ClassElement<'a> {
    let MethodDefinition {
      r#type,
      span,
      decorators,
      key,
      value,
      kind,
      r#static,
      r#override,
      optional,
      accessibility,
      ..
    } = node;

    let (computed, key) = self.transform_property_key(key, true).unwrap();
    let value = self.transform_function(value, true).unwrap();

    self.ast_builder.class_element_method_definition(
      *r#type,
      *span,
      self.clone_node(decorators),
      key,
      value,
      *kind,
      computed,
      *r#static,
      *r#override,
      *optional,
      *accessibility,
    )
  }
}
