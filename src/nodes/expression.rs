use crate::{entity::Entity, TreeShaker};
use oxc::ast::ast::Expression;

#[derive(Debug, Default, Clone)]
pub struct Data {
  included: bool,
  need_val: bool,
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_expression(&mut self, node: &'a Expression, need_val: bool) -> Entity {
    let data = self.load_data::<Data>(node);
    data.included = true;
    data.need_val = need_val;

    match node {
      Expression::NumericLiteral(node) => {
        if need_val {
          self.exec_numeric_literal(node)
        } else {
          Entity::Unknown
        }
      }

      Expression::Identifier(node) => self.exec_identifier_reference_read(node),

      _ => todo!(),
    }
  }
}
