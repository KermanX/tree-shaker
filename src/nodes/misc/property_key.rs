use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{Entity, LiteralCollector, LiteralEntity},
  transformer::Transformer,
};
use oxc::{ast::ast::PropertyKey, span::GetSpan};

const AST_TYPE: AstType2 = AstType2::PropertyKey;

#[derive(Debug)]
pub struct Data<'a> {
  collector: LiteralCollector<'a>,
}

impl<'a> Default for Data<'a> {
  fn default() -> Self {
    Data { collector: LiteralCollector::new_property_key_collector() }
  }
}

impl<'a> Analyzer<'a> {
  pub fn exec_property_key(&mut self, node: &'a PropertyKey<'a>) -> Entity<'a> {
    let entity = match node {
      PropertyKey::StaticIdentifier(node) => LiteralEntity::new_string(node.name.as_str()),
      PropertyKey::PrivateIdentifier(node) => LiteralEntity::new_string(node.name.as_str()),
      node => {
        let node = node.to_expression();
        self.exec_expression(node).get_to_property_key()
      }
    };

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.collector.collect(self, entity)
  }
}

impl<'a> Transformer<'a> {
  /// Returns (computed, node)
  /// Notice that even if `need_val` is `false`, and the expression has side-effect, the transformed expression still evaluates to the original value.
  pub fn transform_property_key(
    &self,
    node: &'a PropertyKey<'a>,
    need_val: bool,
  ) -> Option<(bool, PropertyKey<'a>)> {
    match node {
      // Reuse the node
      PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => {
        need_val.then_some((false, self.clone_node(node)))
      }
      _ => {
        let data = self.get_data::<Data>(AST_TYPE, node);
        let span = node.span();
        let node = node.to_expression();
        if let Some(LiteralEntity::String(s)) = data.collector.collected() {
          let effect = self.transform_expression(node, false);
          if effect.is_some() || need_val {
            let expr = self.transform_expression(node, true).unwrap();
            Some((true, self.ast_builder.property_key_expression(expr)))
          } else if need_val {
            Some((false, self.ast_builder.property_key_identifier_name(span, s)))
          } else {
            None
          }
        } else {
          if need_val || self.transform_expression(node, false).is_some() {
            let expr = self.transform_expression(node, true).unwrap();
            Some((true, self.ast_builder.property_key_expression(expr)))
          } else {
            None
          }
        }
      }
    }
  }
}
