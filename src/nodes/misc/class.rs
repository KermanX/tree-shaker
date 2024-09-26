use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  entity::{Entity, UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{Class, ClassBody, TSTypeParameterDeclaration, TSTypeParameterInstantiation};

impl<'a> Analyzer<'a> {
  pub fn exec_class(&mut self, node: &'a Class<'a>) -> Entity<'a> {
    self.push_variable_scope();

    if node.id.is_some() {
      self.declare_class(node, false);
    }

    let super_class = node.super_class.as_ref().map(|node| self.exec_expression(node));

    super_class.map(|entity| entity.consume(self));

    for element in &node.body.body {
      self.exec_class_element(element);
    }

    self.pop_variable_scope();

    UnknownEntity::new_unknown()
  }

  pub fn declare_class(&mut self, node: &'a Class<'a>, exporting: bool) {
    self.declare_binding_identifier(node.id.as_ref().unwrap(), exporting, DeclarationKind::Class);
  }

  pub fn init_class(&mut self, node: &'a Class<'a>) -> Entity<'a> {
    let value = self.exec_class(node);

    self.init_binding_identifier(node.id.as_ref().unwrap(), Some(value.clone()));

    value
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_class(&self, node: &'a Class<'a>, _need_val: bool) -> Option<Class<'a>> {
    let Class { r#type, span, decorators, id, super_class, body, .. } = node;

    let super_class =
      super_class.as_ref().map(|node| self.transform_expression(node, true).unwrap());

    let body = {
      let ClassBody { span, body, .. } = body.as_ref();

      let mut transformed_body = self.ast_builder.vec();

      for element in body {
        transformed_body.push(self.transform_class_element(element));
      }

      self.ast_builder.class_body(*span, transformed_body)
    };

    Some(self.ast_builder.class(
      *r#type,
      *span,
      self.clone_node(decorators),
      id.clone(),
      None::<TSTypeParameterDeclaration>,
      super_class,
      None::<TSTypeParameterInstantiation>,
      None,
      body,
      false,
      false,
    ))
  }
}
