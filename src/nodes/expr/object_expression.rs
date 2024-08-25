use crate::entity::entity::Entity;
use crate::entity::object::ObjectEntity;
use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_object_expression(&mut self, node: &'a ObjectExpression) -> Entity<'a> {
    let mut object = ObjectEntity::default();

    for property in &node.properties {
      match property {
        ObjectPropertyKind::ObjectProperty(node) => {
          let key = self.exec_property_key(&node.key);
          let value = self.exec_expression(&node.value);
          object.set_property(key, value);
        }
        ObjectPropertyKind::SpreadProperty(node) => todo!(),
      }
    }

    Rc::new(object)
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
