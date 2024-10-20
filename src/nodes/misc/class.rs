use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  consumable::{box_consumable, ConsumableTrait},
  entity::{ClassEntity, Entity},
  transformer::Transformer,
};
use oxc::{
  ast::{
    ast::{Class, ClassBody, ClassElement, MethodDefinitionKind, PropertyKind},
    AstKind, NONE,
  },
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub fn exec_class(&mut self, node: &'a Class<'a>, is_expression: bool) -> Entity<'a> {
    let super_class = node.super_class.as_ref().map(|node| self.exec_expression(node));

    let statics = self.new_empty_object();

    for element in &node.body.body {
      if let ClassElement::MethodDefinition(node) = element {
        if node.r#static {
          let key = self.exec_property_key(&node.key);
          let value = self.exec_function(&node.value, false);
          let kind = match node.kind {
            MethodDefinitionKind::Constructor => unreachable!(),
            MethodDefinitionKind::Method => PropertyKind::Init,
            MethodDefinitionKind::Get => PropertyKind::Get,
            MethodDefinitionKind::Set => PropertyKind::Set,
          };
          statics.init_property(self, kind, key, value, true);
        }
      }
    }

    let class = self.factory.class(
      node,
      is_expression,
      self.scope_context.variable.stack.clone(),
      super_class,
      statics,
    );

    for element in &node.body.body {
      if element.r#static() {
        match element {
          ClassElement::StaticBlock(node) => self.exec_static_block(node, class),
          ClassElement::MethodDefinition(_node) => {}
          ClassElement::PropertyDefinition(node) => {
            if let Some(value) = &node.value {
              let key = self.exec_property_key(&node.key);
              let value = self.exec_expression(value);
              class.set_property(
                self,
                box_consumable(AstKind::PropertyDefinition(node)),
                key,
                value,
              );
            }
          }
          _ => unreachable!(),
        }
      }
    }

    class
  }

  pub fn declare_class(&mut self, node: &'a Class<'a>, exporting: bool) {
    self.declare_binding_identifier(node.id.as_ref().unwrap(), exporting, DeclarationKind::Class);
  }

  pub fn init_class(&mut self, node: &'a Class<'a>) -> Entity<'a> {
    let value = self.exec_class(node, false);

    self.init_binding_identifier(node.id.as_ref().unwrap(), Some(value.clone()));

    value
  }

  pub fn construct_class(&mut self, class: &ClassEntity<'a>) {
    class.super_class.consume(self);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_class(&self, node: &'a Class<'a>, need_val: bool) -> Option<Class<'a>> {
    let Class { r#type, span, id, super_class, body, .. } = node;

    let id = id.as_ref().and_then(|node| self.transform_binding_identifier(node));

    if need_val || id.is_some() {
      let super_class =
        super_class.as_ref().map(|node| self.transform_expression(node, true).unwrap());

      let body = {
        let ClassBody { span, body, .. } = body.as_ref();

        let mut transformed_body = self.ast_builder.vec();

        for element in body {
          if let Some(element) = match element {
            ClassElement::StaticBlock(node) => self
              .transform_static_block(node)
              .map(|node| self.ast_builder.class_element_from_static_block(node)),
            ClassElement::MethodDefinition(node) => self.transform_method_definition(node),
            ClassElement::PropertyDefinition(node) => self.transform_property_definition(node),
            ClassElement::AccessorProperty(_node) => unreachable!(),
            ClassElement::TSIndexSignature(_node) => unreachable!(),
          } {
            transformed_body.push(element);
          }
        }

        self.ast_builder.class_body(*span, transformed_body)
      };

      Some(self.ast_builder.class(
        *r#type,
        *span,
        self.ast_builder.vec(),
        id.clone(),
        NONE,
        super_class,
        NONE,
        None,
        body,
        false,
        false,
      ))
    } else {
      let mut statements = self.ast_builder.vec();

      if let Some(super_class) = super_class {
        let span = super_class.span();
        if let Some(super_class) = self.transform_expression(super_class, false) {
          statements.push(self.ast_builder.statement_expression(span, super_class));
        }
      }

      for element in &body.body {
        if let Some(key) = element.property_key() {
          if key.is_expression() {
            if let Some(element) = self.transform_expression(key.to_expression(), false) {
              statements.push(self.ast_builder.statement_expression(element.span(), element));
            }
          }
        }
      }

      for element in &body.body {
        match element {
          ClassElement::StaticBlock(node) => {
            if let Some(node) = self.transform_static_block(node) {
              statements.push(self.ast_builder.statement_block(node.span, node.body));
            }
          }
          ClassElement::PropertyDefinition(node) if node.r#static => {
            if let Some(value) = &node.value {
              let span = value.span();
              if let Some(value) = self.transform_expression(value, false) {
                statements.push(self.ast_builder.statement_expression(span, value));
              }
            }
          }
          _ => {}
        }
      }

      if statements.is_empty() {
        None
      } else {
        Some(
          self.ast_builder.class(
            *r#type,
            *span,
            self.ast_builder.vec(),
            None,
            NONE,
            None,
            NONE,
            None,
            self.ast_builder.class_body(
              body.span(),
              self
                .ast_builder
                .vec1(self.ast_builder.class_element_static_block(body.span(), statements)),
            ),
            false,
            false,
          ),
        )
      }
    }
  }
}
