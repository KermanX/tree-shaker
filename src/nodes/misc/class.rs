use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  consumable::{box_consumable, ConsumableTrait},
  entity::{ClassEntity, Entity, FunctionEntitySource},
  transformer::Transformer,
};
use oxc::{
  ast::{
    ast::{
      Class, ClassBody, ClassElement, MethodDefinitionKind, PropertyDefinitionType, PropertyKind,
    },
    AstKind, NONE,
  },
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub fn exec_class(&mut self, node: &'a Class<'a>) -> Entity<'a> {
    let super_class = node.super_class.as_ref().map(|node| self.exec_expression(node));

    let mut keys = vec![];
    for element in &node.body.body {
      keys.push(element.property_key().map(|key| self.exec_property_key(key)));
    }

    let statics = self.new_empty_object(&self.builtins.prototypes.function);
    for (index, element) in node.body.body.iter().enumerate() {
      if let ClassElement::MethodDefinition(node) = element {
        if node.r#static {
          let key = keys[index].unwrap();
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
      keys.clone(),
      self.scope_context.variable.stack.clone(),
      super_class,
      statics,
    );

    for (index, element) in node.body.body.iter().enumerate() {
      match element {
        ClassElement::StaticBlock(node) => self.exec_static_block(node, class),
        ClassElement::MethodDefinition(_node) => {}
        ClassElement::PropertyDefinition(node) if node.r#static => {
          if let Some(value) = &node.value {
            let key = keys[index].unwrap();
            let value = self.exec_expression(value);
            class.set_property(self, box_consumable(AstKind::PropertyDefinition(node)), key, value);
          }
        }
        _ => {}
      }
    }

    class
  }

  pub fn declare_class(&mut self, node: &'a Class<'a>, exporting: bool) {
    self.declare_binding_identifier(node.id.as_ref().unwrap(), exporting, DeclarationKind::Class);
  }

  pub fn init_class(&mut self, node: &'a Class<'a>) -> Entity<'a> {
    let value = self.exec_class(node);

    self.init_binding_identifier(node.id.as_ref().unwrap(), Some(value.clone()));

    value
  }

  pub fn construct_class(&mut self, class: &ClassEntity<'a>) {
    let node = class.node;

    self.consume(AstKind::Class(node));

    class.super_class.consume(self);

    // Keys
    for (index, element) in node.body.body.iter().enumerate() {
      if !element.r#static() {
        if let Some(key) = class.keys[index] {
          key.consume(self);
        }
      }
    }

    // Non-static methods
    for element in &node.body.body {
      if let ClassElement::MethodDefinition(node) = element {
        if !node.r#static {
          let value = self.exec_function(&node.value, true);
          self.consume(value);
        }
      }
    }

    // Non-static properties
    let variable_scope_stack = class.variable_scope_stack.clone();
    self.exec_consumed_fn(move |analyzer| {
      analyzer.push_call_scope(
        FunctionEntitySource::ClassConstructor(node),
        box_consumable(()),
        variable_scope_stack.as_ref().clone(),
        analyzer.factory.unknown,
        (analyzer.factory.unknown, vec![]),
        false,
        false,
        false,
      );

      for element in &node.body.body {
        if let ClassElement::PropertyDefinition(node) = element {
          if !node.r#static {
            if let Some(value) = &node.value {
              let value = analyzer.exec_expression(value);
              analyzer.consume(value);
            }
          }
        }
      }

      analyzer.pop_call_scope();

      analyzer.factory.undefined
    });
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_class(&self, node: &'a Class<'a>, need_val: bool) -> Option<Class<'a>> {
    let Class { r#type, span, id, super_class, body, .. } = node;

    if need_val || id.is_some() {
      let id = if self.config.preserve_function_name {
        self.clone_node(id)
      } else {
        id.as_ref().and_then(|node| self.transform_binding_identifier(node))
      };

      let ever_constructed = self.is_referred(AstKind::Class(node));

      let super_class = super_class.as_ref().and_then(|node| {
        if ever_constructed || self.transform_expression(node, false).is_some() {
          self.transform_expression(node, true)
        } else {
          None
        }
      });

      let body = {
        let ClassBody { span, body, .. } = body.as_ref();

        let mut transformed_body = self.ast_builder.vec();

        for element in body {
          if ever_constructed || element.r#static() {
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
          } else if let Some(key) =
            element.property_key().and_then(|key| self.transform_property_key(key, false))
          {
            transformed_body.push(self.ast_builder.class_element_property_definition(
              PropertyDefinitionType::PropertyDefinition,
              element.span(),
              self.ast_builder.vec(),
              key,
              None,
              false,
              false,
              false,
              false,
              false,
              false,
              false,
              NONE,
              None,
            ));
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
