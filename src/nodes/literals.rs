use crate::{entity::Entity, TreeShaker};
use oxc::ast::ast::NumericLiteral;

#[derive(Debug, Default, Clone)]
pub struct Data {
  included: bool,
}

impl TreeShaker<'_> {
  pub(crate) fn exec_numeric_literal(&mut self, node: &NumericLiteral) -> Entity {
    let data = self.load_data::<Data>(node);
    data.included = true;
    Entity::NumberLiteral(node.value)
  }
}
