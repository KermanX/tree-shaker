use crate::{
  analyzer::Analyzer, ast::AstKind2, consumable::box_consumable, entity::ObjectEntity,
  transformer::Transformer,
};
use oxc::{
  allocator,
  ast::ast::{Expression, JSXAttribute, JSXAttributeItem, JSXSpreadAttribute, PropertyKind},
};

impl<'a> Analyzer<'a> {
  pub fn exec_jsx_attributes(
    &mut self,
    node: &'a allocator::Vec<'a, JSXAttributeItem<'a>>,
  ) -> ObjectEntity<'a> {
    let object = self.new_empty_object(&self.builtins.prototypes.object);

    for attr in node.iter() {
      let dep_id = AstKind2::JSXAttributeItem(attr);
      match attr {
        JSXAttributeItem::Attribute(node) => {
          let key = self.exec_jsx_attribute_name(&node.name);
          let value = self.factory.computed(self.exec_jsx_attribute_value(&node.value), dep_id);
          object.init_property(self, PropertyKind::Init, key, value, true);
        }
        JSXAttributeItem::SpreadAttribute(node) => {
          let argument = self.exec_expression(&node.argument);
          object.init_spread(self, box_consumable(dep_id), argument);
        }
      }
    }

    object
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_jsx_attributes_effect_only(
    &self,
    node: &'a allocator::Vec<'a, JSXAttributeItem<'a>>,
  ) -> Vec<Expression<'a>> {
    node
      .iter()
      .filter_map(|attr| match attr {
        JSXAttributeItem::Attribute(node) => {
          self.transform_jsx_attribute_value_effect_only(&node.value)
        }
        JSXAttributeItem::SpreadAttribute(node) => {
          let JSXSpreadAttribute { span, argument } = node.as_ref();

          if self.is_referred(AstKind2::JSXAttributeItem(attr)) {
            let argument = self.transform_expression(argument, true).unwrap();
            Some(self.build_object_spread_effect(*span, argument))
          } else {
            self.transform_expression(argument, false)
          }
        }
      })
      .collect()
  }

  pub fn transform_jsx_attributes_need_val(
    &self,
    node: &'a allocator::Vec<'a, JSXAttributeItem<'a>>,
  ) -> allocator::Vec<'a, JSXAttributeItem<'a>> {
    let mut transformed = self.ast_builder.vec_with_capacity(node.len());

    for attr in node.iter() {
      let is_referred = self.is_referred(AstKind2::JSXAttributeItem(attr));
      match attr {
        JSXAttributeItem::Attribute(node) => {
          let JSXAttribute { span, name, value } = node.as_ref();

          if let Some(value) = self.transform_jsx_attribute_value_as_item(value, is_referred) {
            transformed.push(self.ast_builder.jsx_attribute_item_jsx_attribute(
              *span,
              self.clone_node(name),
              Some(value),
            ));
          }
        }
        JSXAttributeItem::SpreadAttribute(node) => {
          let JSXSpreadAttribute { span, argument } = node.as_ref();

          if is_referred {
            transformed.push(self.ast_builder.jsx_attribute_item_jsx_spread_attribute(
              *span,
              self.transform_expression(argument, true).unwrap(),
            ))
          }
        }
      }
    }

    transformed
  }
}
