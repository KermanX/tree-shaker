use crate::build_effect;
use crate::entity::entity::Entity;
use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::{
  Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind, SpreadElement,
};
use oxc::span::{GetSpan, SPAN};
use std::rc::Rc;

#[derive(Debug, Default)]
struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_object_expression(&mut self, node: &'a ObjectExpression) -> Entity<'a> {
    let object = self.new_empty_object();

    for property in &node.properties {
      match property {
        ObjectPropertyKind::ObjectProperty(node) => {
          let key = self.exec_property_key(&node.key);
          let value = self.exec_expression(&node.value);
          object.init_property(node.kind, key, value, true);
        }
        ObjectPropertyKind::SpreadProperty(node) => {
          let argument = self.exec_expression(&node.argument);
          let has_effect = object.init_spread(self, argument);

          let data = self.load_data::<Data>(node.as_ref());
          data.has_effect |= has_effect;
        }
      }
    }

    Rc::new(object)
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
          let ObjectProperty { span, key, kind, value, method, .. } = node.as_ref();

          let value_span = value.span();

          let value = self.transform_expression(value, need_val);

          if let Some(value) = value {
            let (computed, key) = self.transform_property_key(key, true).unwrap();
            self.ast_builder.object_property_kind_object_property(
              *span, *kind, key, value, None, *method, false, computed,
            )
          } else {
            if let Some((computed, key)) = self.transform_property_key(key, false) {
              self.ast_builder.object_property_kind_object_property(
                *span,
                *kind,
                key,
                self.build_unused_expression(value_span),
                None,
                *method,
                false,
                computed,
              )
            } else {
              continue;
            }
          }
        }
        ObjectPropertyKind::SpreadProperty(node) => {
          let data = self.get_data::<Data>(node.as_ref());
          let need_spread = need_val || data.has_effect;

          let SpreadElement { span, argument, .. } = node.as_ref();

          let argument = self.transform_expression(argument, need_spread);

          if let Some(argument) = argument {
            self.ast_builder.object_property_kind_spread_element(
              *span,
              if need_spread {
                argument
              } else {
                build_effect!(&self.ast_builder, *span, Some(argument); self.build_unused_expression(SPAN))
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
