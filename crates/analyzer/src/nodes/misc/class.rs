use crate::{host::Host, 
  analyzer::Analyzer,
  ast::{AstKind2, DeclarationKind},
  consumable::ConsumableTrait,
  entity::{ClassEntity, Entity},
    utils::CalleeNode,
};
use oxc::{
  allocator,
  ast::{
    ast::{
      Class, ClassBody, ClassElement, ClassType, MethodDefinitionKind, PropertyDefinitionType,
      PropertyKind, StaticBlock,
    },
    NONE,
  },
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_class(&mut self, node: &'a Class<'a>) -> H::Entity {
    let super_class = node.super_class.as_ref().map(|node| self.exec_expression(node));

    let mut keys = vec![];
    for element in &node.body.body {
      keys.push(element.property_key().map(|key| self.exec_property_key(key)));
    }

    self.push_variable_scope();

    let statics = self.new_empty_object(&self.builtins.prototypes.function, None);
    for (index, element) in node.body.body.iter().enumerate() {
      if let ClassElement::MethodDefinition(node) = element {
        if node.r#static {
          let key = keys[index].unwrap();
          let value = self.exec_function(&node.value);
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

    self.pop_variable_scope();

    let class = self.factory.class(
      node,
      keys.clone(),
      self.scope_context.variable.stack.clone(),
      super_class,
      statics,
    );

    let variable_scope_stack = self.scope_context.variable.stack.clone();
    self.push_call_scope(
      self.new_callee_info(CalleeNode::ClassStatics(node)),
      self.factory.empty_consumable,
      variable_scope_stack,
      false,
      false,
      false,
    );

    let variable_scope = self.variable_scope_mut();
    variable_scope.this = Some(class);

    if let Some(id) = &node.id {
      self.declare_binding_identifier(id, false, DeclarationKind::NamedFunctionInBody);
      self.init_binding_identifier(id, Some(class));
    }

    for (index, element) in node.body.body.iter().enumerate() {
      match element {
        ClassElement::StaticBlock(node) => self.exec_static_block(node),
        ClassElement::MethodDefinition(_node) => {}
        ClassElement::PropertyDefinition(node) if node.r#static => {
          if let Some(value) = &node.value {
            let key = keys[index].unwrap();
            let value = self.exec_expression(value);
            class.set_property(
              self,
              self.consumable(AstKind2::PropertyDefinition(node)),
              key,
              value,
            );
          }
        }
        _ => {}
      }
    }

    self.pop_call_scope();

    class
  }

  pub fn declare_class(&mut self, node: &'a Class<'a>, exporting: bool) {
    self.declare_binding_identifier(node.id.as_ref().unwrap(), exporting, DeclarationKind::Class);
  }

  pub fn init_class(&mut self, node: &'a Class<'a>) -> H::Entity {
    let value = self.exec_class(node);

    self.init_binding_identifier(node.id.as_ref().unwrap(), Some(value));

    value
  }

  pub fn construct_class(&mut self, class: &ClassEntity<'a>) {
    let node = class.node;

    self.consume(AstKind2::Class(node));

    class.super_class.consume(self);

    // Keys
    for (index, element) in node.body.body.iter().enumerate() {
      if !element.r#static() {
        if let Some(key) = class.keys[index] {
          key.consume(self);
        }
      }
    }

    if let Some(id) = &node.id {
      self.push_variable_scope();
      self.declare_binding_identifier(id, false, DeclarationKind::NamedFunctionInBody);
      self.init_binding_identifier(id, Some(self.factory.unknown()));
    }

    // Non-static methods
    for element in &node.body.body {
      if let ClassElement::MethodDefinition(node) = element {
        if !node.r#static {
          let value = self.exec_function(&node.value);
          self.consume(value);
        }
      }
    }

    // Non-static properties
    let variable_scope_stack = class.variable_scope_stack.clone();
    self.exec_consumed_fn("class_property", move |analyzer| {
      analyzer.push_call_scope(
        analyzer.new_callee_info(CalleeNode::ClassConstructor(node)),
        analyzer.factory.empty_consumable,
        variable_scope_stack.as_ref().clone(),
        false,
        false,
        false,
      );

      let this = analyzer.factory.unknown();
      let arguments = analyzer.factory.immutable_unknown;
      let variable_scope = analyzer.variable_scope_mut();
      variable_scope.this = Some(this);
      variable_scope.arguments = Some((arguments, vec![]));

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

    if node.id.is_some() {
      self.pop_variable_scope();
    }
  }
}

