use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{Entity, LiteralCollector, LiteralEntity},
  transformer::Transformer,
};
use oxc::{ast::ast::PropertyKey, span::GetSpan};

const AST_TYPE: AstType2 = AstType2::PropertyKey;

#[derive(Debug, Default)]
pub struct Data<'a> {
  collector: LiteralCollector<'a>,
}

impl<'a> Analyzer<'a> {
  pub fn exec_property_key(&mut self, node: &'a PropertyKey<'a>) -> Entity<'a> {
    let entity = match node {
      PropertyKey::StaticIdentifier(node) => LiteralEntity::new_string(node.name.as_str()),
      PropertyKey::PrivateIdentifier(node) => LiteralEntity::new_string(node.name.as_str()),
      node => {
        let node = node.to_expression();
        self.exec_expression(node)
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
        if let Some((r#static, s)) = data.collector.collected_property_key(&self.config) {
          let effect = self.transform_expression(node, false);
          if effect.is_some() || need_val {
            let expr = self.transform_expression(node, true).unwrap();
            Some((true, self.ast_builder.property_key_expression(expr)))
          } else if need_val {
            if r#static {
              Some((false, self.ast_builder.property_key_identifier_name(span, s)))
            } else {
              Some((
                false,
                self
                  .ast_builder
                  .property_key_expression(self.ast_builder.expression_string_literal(span, s)),
              ))
            }
          } else {
            None
          }
        } else {
          let expr = self.transform_expression(node, true).unwrap();
          Some((true, self.ast_builder.property_key_expression(expr)))
        }
      }
    }
  }
}
