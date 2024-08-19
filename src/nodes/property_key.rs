use crate::{entity::Entity, TreeShakerImpl};
use oxc::ast::ast::PropertyKey;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> TreeShakerImpl<'a> {
  pub(crate) fn exec_property_key(&mut self, node: &'a PropertyKey) -> Entity {
    let data = self.load_data::<Data>(node);

    match node {
      PropertyKey::Identifier(node) => Entity::StringLiteral(node.name.clone().into_string()),
      PropertyKey::PrivateIdentifier(node) => todo!(),
      node => {
        let node = node.to_expression();
        self.exec_expression(node)
      }
    }
  }
}
