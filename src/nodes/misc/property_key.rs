use crate::{
  analyzer::Analyzer, ast::AstKind2, entity::Entity, mangling::MangleAtom, transformer::Transformer,
};
use oxc::{
  ast::ast::{IdentifierName, PrivateIdentifier, PropertyKey},
  span::{Atom, GetSpan},
};

impl<'a> Analyzer<'a> {
  pub fn exec_property_key(&mut self, node: &'a PropertyKey<'a>) -> Entity<'a> {
    match node {
      PropertyKey::StaticIdentifier(static_identifier) => {
        self.exec_static_property_key(node, static_identifier.name.as_str())
      }
      PropertyKey::PrivateIdentifier(private_identifier) => {
        // FIXME: Not good
        self.exec_static_property_key(
          node,
          self.escape_private_identifier_name(private_identifier.name.as_str()),
        )
      }
      node => self.exec_expression(node.to_expression()).get_to_property_key(self),
    }
  }

  fn exec_static_property_key(&mut self, node: &'a PropertyKey<'a>, name: &'a str) -> Entity<'a> {
    let atom = *self.get_data_or_insert_with::<MangleAtom>(AstKind2::PropertyKey(node), || {
      MangleAtom::new(node.span())
    });
    self.factory.mangable_string(name, atom)
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

    let resolve_mangled_name = |original: &'a Atom<'a>| {
      let atom = *self.force_get_data::<MangleAtom>(AstKind2::PropertyKey(node));
      let mut mangler = self.mangler.borrow_mut();
      mangler.resolve(atom).unwrap_or(original.as_str())
    };

    match node {
      // Reuse the node
      PropertyKey::StaticIdentifier(node) => {
        let IdentifierName { span, name } = node.as_ref();
        need_val
          .then(|| self.ast_builder.property_key_identifier_name(*span, resolve_mangled_name(name)))
      }
      PropertyKey::PrivateIdentifier(private_identifier) => {
        let PrivateIdentifier { span, name } = private_identifier.as_ref();
        need_val.then(|| {
          self.ast_builder.property_key_private_identifier(*span, resolve_mangled_name(name))
        })
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
