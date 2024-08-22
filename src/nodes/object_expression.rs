use crate::{
  analyzer::Analyzer,
  entity::{object::ObjectEntity, Entity},
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_object_expression(&mut self, node: &'a ObjectExpression) -> (bool, Entity) {
    let mut effect = false;
    let mut value = ObjectEntity::default();

    for property in &node.properties {
      match property {
        ObjectPropertyKind::ObjectProperty(node) => {
          let (key_effect, key_val) = self.exec_property_key(&node.key);
          let (value_effect, value_val) = self.exec_expression(&node.value);
          effect |= key_effect || value_effect;
          value.init_property(&key_val, value_val.clone());
        }
        ObjectPropertyKind::SpreadProperty(node) => todo!(),
      }
    }

    (effect, Entity::Object(value))
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_object_expression(
    &self,
    node: ObjectExpression<'a>,
  ) -> Option<Expression<'a>> {
    // TODO:  !!!!
    Some(self.ast_builder.expression_from_object(node))
  }
}
