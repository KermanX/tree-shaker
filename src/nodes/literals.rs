use crate::{entity::Entity, TreeShakerImpl};
use oxc::ast::ast::NumericLiteral;

#[derive(Debug, Default, Clone)]
pub struct Data {
  included: bool,
}

impl<'a> TreeShakerImpl<'a> {
  pub(crate) fn exec_numeric_literal(&mut self, node: &'a NumericLiteral) -> Entity {
    let data = self.load_data::<Data>(node);
    data.included = true;
    Entity::NumberLiteral(node.value)
  }
}
