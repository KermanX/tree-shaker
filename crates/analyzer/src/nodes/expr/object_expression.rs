use crate::{host::Host, 
  analyzer::Analyzer,
  
  
  entity::{Entity, EntityTrait},
  };
use oxc::{
  ast::ast::{
    Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyKey, PropertyKind,
    SpreadElement,
  },
  span::{GetSpan, SPAN},
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_object_expression(&mut self, node: &'a ObjectExpression) -> H::Entity {
    let object = self.use_mangable_plain_object(AstKind2::ObjectExpression(node));

    let mut has_proto = false;

    for property in &node.properties {
      match property {
        ObjectPropertyKind::ObjectProperty(node) => {
          let key = self.exec_property_key(&node.key);
          let value = self.exec_expression(&node.value);
          let value = self.factory.computed(value, AstKind2::ObjectProperty(node));

          if matches!(&node.key, PropertyKey::StaticIdentifier(node) if node.name == "__proto__") {
            has_proto = true;
            // Ensure the __proto__ is consumed - it may be overridden by the next property like ["__proto__"]: 1
            self.consume((key, value));
          } else {
            object.init_property(self, node.kind, key, value, true);
          }
        }
        ObjectPropertyKind::SpreadProperty(node) => {
          let argument = self.exec_expression(&node.argument);
          object.init_spread(self, self.consumable(AstKind2::SpreadElement(node)), argument);
        }
      }
    }

    if has_proto {
      // Deoptimize the object
      object.consume(self);
    }

    object
  }
}

