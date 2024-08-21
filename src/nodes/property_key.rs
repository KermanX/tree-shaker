use crate::{entity::Entity, transformer::Transformer, Analyzer};
use oxc::{ast::ast::PropertyKey, span::GetSpan};

#[derive(Debug, Default, Clone)]
pub struct Data {
  value: Entity,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_property_key(&mut self, node: &'a PropertyKey) -> (bool, Entity) {
    let (effect, value) = match node {
      PropertyKey::StaticIdentifier(node) => {
        (false, Entity::StringLiteral(node.name.clone().into_string()))
      }
      PropertyKey::PrivateIdentifier(node) => todo!(),
      node => {
        let node = node.to_expression();
        let (effect, value) = self.exec_expression(node);
        (effect, value.to_property_key())
      }
    };

    self.set_data(node, Data { value: value.clone() });

    (effect, value)
  }

  pub(crate) fn calc_property_key(&self, node: &'a PropertyKey) -> Entity {
    let data = self.get_data::<Data>(node);

    data.value.clone()
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_property_key(
    &self,
    node: PropertyKey<'a>,
    need_val: bool,
  ) -> Option<PropertyKey<'a>> {
    let data = self.get_data::<Data>(&node);

    match node {
      PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => Some(node),
      _ => match &data.value {
        Entity::StringLiteral(s) => {
          let span = node.span();
          self.transform_expression(TryFrom::try_from(node).unwrap(), false);
          Some(self.ast_builder.property_key_identifier_name(span, s))
        }
        _ => {
          let expr = self.transform_expression(node.try_into().unwrap(), need_val);
          expr.map(|e| self.ast_builder.property_key_expression(e))
        }
      },
    }
  }
}
