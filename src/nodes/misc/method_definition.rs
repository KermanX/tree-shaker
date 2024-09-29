use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::{
  ast::ast::{ClassElement, Function, MethodDefinition, MethodDefinitionKind},
  span::SPAN,
};

impl<'a> Analyzer<'a> {
  pub fn exec_method_definition(&mut self, node: &'a MethodDefinition<'a>) {
    let value = self.exec_function(&node.value, true);
    self.consume(value);
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
    let mut value = self.transform_function(value, true).unwrap();

    if *kind == MethodDefinitionKind::Set {
      self.patch_method_definition_params(&mut value);
    }

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

  /// It is possible that `set a(param) {}` has been optimized to `set a() {}`.
  /// This function patches the parameter list if it is empty.
  pub fn patch_method_definition_params(&self, node: &mut Function<'a>) {
    if !node.params.has_parameter() {
      node.params.items.push(self.ast_builder.formal_parameter(
        SPAN,
        self.ast_builder.vec(),
        self.build_unused_binding_pattern(SPAN),
        None,
        false,
        false,
      ));
    }
  }
}
