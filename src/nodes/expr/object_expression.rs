use crate::{
  analyzer::Analyzer,
  ast::AstKind2,
  build_effect,
  consumable::box_consumable,
  entity::{Entity, EntityTrait},
  transformer::Transformer,
};
use oxc::{
  ast::ast::{
    Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyKey, PropertyKind,
    SpreadElement,
  },
  span::{GetSpan, SPAN},
};

impl<'a> Analyzer<'a> {
  pub fn exec_object_expression(&mut self, node: &'a ObjectExpression) -> Entity<'a> {
    let object = self.new_empty_object(&self.builtins.prototypes.object);

    let mut has_proto = false;

    for property in &node.properties {
      match property {
        ObjectPropertyKind::ObjectProperty(node) => {
          let key = self.exec_property_key(&node.key);
          let value = self.exec_expression(&node.value);
          let value = self.factory.computed(value, AstKind2::ObjectProperty(node));

          match &node.key {
            PropertyKey::StaticIdentifier(node) if node.name == "__proto__" => {
              has_proto = true;
              // Ensure the __proto__ is consumed - it may be overridden by the next property like ["__proto__"]: 1
              self.consume(value);
            }
            _ => {
              object.init_property(self, node.kind, key, value, true);
            }
          };
        }
        ObjectPropertyKind::SpreadProperty(node) => {
          let argument = self.exec_expression(&node.argument);
          object.init_spread(self, box_consumable(AstKind2::SpreadElement(node)), argument);
        }
      }
    }

    if has_proto {
      // Deoptimize the object
      object.consume(self);
    }

    self.factory.entity(object)
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_object_expression(
    &self,
    node: &'a ObjectExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let ObjectExpression { span, properties, .. } = node;

    let mut transformed_properties = self.ast_builder.vec();

    for property in properties {
      transformed_properties.push(match property {
        ObjectPropertyKind::ObjectProperty(node) => {
          let ObjectProperty { span, key, kind, value, method, computed, .. } = node.as_ref();

          let value_span = value.span();

          let transformed_value =
            self.transform_expression(value, self.is_referred(AstKind2::ObjectProperty(node)));

          if let Some(mut transformed_value) = transformed_value {
            if *kind == PropertyKind::Set {
              if let (
                Expression::FunctionExpression(original_node),
                Expression::FunctionExpression(transformed_node),
              ) = (value, &mut transformed_value)
              {
                self.patch_method_definition_params(original_node, transformed_node);
              } else {
                unreachable!()
              }
            }

            let key = self.transform_property_key(key, true).unwrap();
            self.ast_builder.object_property_kind_object_property(
              *span,
              *kind,
              key,
              transformed_value,
              None,
              *method,
              false,
              *computed,
            )
          } else {
            if let Some(key) = self.transform_property_key(key, false) {
              self.ast_builder.object_property_kind_object_property(
                *span,
                *kind,
                key,
                self.build_unused_expression(value_span),
                None,
                *method,
                false,
                *computed,
              )
            } else {
              continue;
            }
          }
        }
        ObjectPropertyKind::SpreadProperty(node) => {
          let SpreadElement { span, argument, .. } = node.as_ref();

          let referred = self.is_referred(AstKind2::SpreadElement(node));

          let argument = self.transform_expression(argument, referred);

          if let Some(argument) = argument {
            self.ast_builder.object_property_kind_spread_element(
              *span,
              if referred {
                argument
              } else {
                build_effect!(
                  &self.ast_builder,
                  *span,
                  Some(argument);
                  self.ast_builder.expression_object(SPAN, self.ast_builder.vec(), None)
                )
              },
            )
          } else {
            continue;
          }
        }
      });
    }

    if !need_val && transformed_properties.is_empty() {
      None
    } else {
      Some(self.ast_builder.expression_object(*span, transformed_properties, None))
    }
  }
}
