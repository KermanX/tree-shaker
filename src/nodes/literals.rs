use crate::{entity::Entity, TreeShaker};
use oxc::ast::ast::{BooleanLiteral, NumericLiteral, StringLiteral};

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exc_numeric_literal(&mut self, node: &'a NumericLiteral) -> Entity {
    let data = self.load_data::<Data>(node);
    Entity::NumberLiteral(node.value)
  }

  pub(crate) fn exec_string_literal(&mut self, node: &'a StringLiteral) -> Entity {
    let data = self.load_data::<Data>(node);
    Entity::StringLiteral(node.value.to_string())
  }

  pub(crate) fn exec_boolean_literal(&mut self, node: &'a BooleanLiteral) -> Entity {
    let data = self.load_data::<Data>(node);
    Entity::BooleanLiteral(node.value)
  }
}
