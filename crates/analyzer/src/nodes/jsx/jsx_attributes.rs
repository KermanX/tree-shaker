use crate::{host::Host, analyzer::Analyzer,  entity::ObjectEntity};
use oxc::{
  allocator,
  ast::ast::{
    Expression, JSXAttribute, JSXAttributeItem, JSXOpeningElement, JSXSpreadAttribute, PropertyKind,
  },
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_attributes(
    &mut self,
    node: &'a JSXOpeningElement<'a>,
  ) -> &'a mut ObjectEntity<'a> {
    let object = self.use_mangable_plain_object(AstKind2::JSXOpeningElement(node));

    for attr in &node.attributes {
      let dep_id = AstKind2::JSXAttributeItem(attr);
      match attr {
        JSXAttributeItem::Attribute(node) => {
          let key = self.exec_jsx_attribute_name(&node.name);
          let value = self.factory.computed(self.exec_jsx_attribute_value(&node.value), dep_id);
          object.init_property(self, PropertyKind::Init, key, value, true);
        }
        JSXAttributeItem::SpreadAttribute(node) => {
          let argument = self.exec_expression(&node.argument);
          object.init_spread(self, self.consumable(dep_id), argument);
        }
      }
    }

    object
  }
}

