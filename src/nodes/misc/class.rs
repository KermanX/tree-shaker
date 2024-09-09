use crate::{
  analyzer::Analyzer,
  entity::{entity::Entity, unknown::UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{
  Class, ClassBody, TSTypeParameterDeclaration, TSTypeParameterInstantiation,
  VariableDeclarationKind,
};

impl<'a> Analyzer<'a> {
  pub fn exec_class(&mut self, node: &'a Class<'a>, exporting: bool) -> Entity<'a> {
    let super_class = node.super_class.as_ref().map(|node| self.exec_expression(node));

    for element in &node.body.body {
      self.exec_class_element(element);
    }

    if let Some(id) = &node.id {
      self.exec_binding_identifier(
        id,
        UnknownEntity::new_unknown(),
        exporting,
        VariableDeclarationKind::Let,
      );
    }

    super_class.map(|entity| entity.consume_as_unknown(self));

    UnknownEntity::new_unknown()
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
