use crate::entity::entity::Entity;
use crate::entity::object::ObjectEntity;
use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::{Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind};
use oxc::span::GetSpan;
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_object_expression(&mut self, node: &'a ObjectExpression) -> Entity<'a> {
    let mut object = ObjectEntity::new_empty();

    for property in &node.properties {
      match property {
        ObjectPropertyKind::ObjectProperty(node) => {
          let key = self.exec_property_key(&node.key);
          let value = self.exec_expression(&node.value);
          object.init_property(key, value);
        }
        ObjectPropertyKind::SpreadProperty(node) => {
          let argument = self.exec_expression(&node.argument);
          object.init_spread(argument)
        }
      }
    }

    Rc::new(object)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_object_expression(
    &mut self,
    node: ObjectExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let ObjectExpression { span, properties, .. } = node;

    let mut transformed_properties = self.ast_builder.vec();

    for property in properties {
      transformed_properties.push(match property {
        ObjectPropertyKind::ObjectProperty(node) => {
          let ObjectProperty { span, key, kind, value, method, .. } = node.unbox();

          let value_span = value.span();

          let value = self.transform_expression(value, need_val);

          if let Some(value) = value {
            let (computed, key) = self.transform_property_key(key, true).unwrap();
            self.ast_builder.object_property_kind_object_property(
              span, kind, key, value, None, method, false, computed,
            )
          } else {
            if let Some((computed, key)) = self.transform_property_key(key, false) {
              self.ast_builder.object_property_kind_object_property(
                span,
                kind,
                key,
                self.build_unused_expression(value_span),
                None,
                method,
                false,
                computed,
              )
            } else {
              continue;
            }
          }
        }
        ObjectPropertyKind::SpreadProperty(node) => todo!(),
      });
    }

    if !need_val && transformed_properties.is_empty() {
      None
    } else {
      Some(self.ast_builder.expression_object(span, transformed_properties, None))
    }
  }
}
