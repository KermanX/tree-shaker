use crate::{entity::Entity, TreeShaker};
use oxc::ast::ast::PropertyKey;

#[derive(Debug, Default, Clone)]
pub struct Data {
  included: bool,
  need_val: bool,
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_property_key(&mut self, node: &'a PropertyKey, need_val: bool) -> Entity {
    let data = self.load_data::<Data>(node);
    data.included = true;
    data.need_val = need_val;

    match node {
      PropertyKey::Identifier(node) => Entity::StringLiteral(node.name.clone().into_string()),
      PropertyKey::PrivateIdentifier(node) => todo!(),
      node => {
        let node = node.to_expression();
        self.exec_expression(node, need_val)
      }
    }
  }
}
