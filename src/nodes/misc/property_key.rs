use crate::{analyzer::Analyzer, entity::Entity, transformer::Transformer};
use oxc::{ast::ast::PropertyKey, span::GetSpan};

impl<'a> Analyzer<'a> {
  pub fn exec_property_key(&mut self, node: &'a PropertyKey<'a>) -> Entity<'a> {
    match node {
      PropertyKey::StaticIdentifier(node) => self.factory.string(node.name.as_str()),
      PropertyKey::PrivateIdentifier(node) => {
        // FIXME: Not good
        self.factory.string(self.escape_private_identifier_name(node.name.as_str()))
      }
      node => self.exec_expression(node.to_expression()).get_to_property_key(self),
    }
  }
}

impl<'a> Transformer<'a> {
  /// Returns (computed, node)
  /// Notice that even if `need_val` is `false`, and the expression has side-effect, the transformed expression still evaluates to the original value.
  pub fn transform_property_key(
    &self,
    node: &'a PropertyKey<'a>,
    need_val: bool,
  ) -> Option<PropertyKey<'a>> {
    if self.declaration_only.get() {
      return need_val.then_some(PropertyKey::from(self.build_unused_expression(node.span())));
    }

    match node {
      // Reuse the node
      PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => {
        need_val.then_some(self.clone_node(node))
      }
      _ => {
        let node = node.to_expression();
        if need_val || self.transform_expression(node, false).is_some() {
          let expr = self.transform_expression(node, true).unwrap();
          Some(PropertyKey::from(expr))
        } else {
          None
        }
      }
    }
  }
}
