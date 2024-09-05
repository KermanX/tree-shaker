use crate::ast::AstType2;
use crate::build_effect;
use crate::entity::collector::LiteralCollector;
use crate::entity::entity::Entity;
use crate::entity::literal::LiteralEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::span::SPAN;
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
    data.collector.collect(entity)
  }
}

impl<'a> Transformer<'a> {
  /// Returns (computed, node)
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
        if let Some(LiteralEntity::String(s)) = data.collector.collected() {
          need_val.then(|| {
            let span = node.span();
            let expr = self.transform_expression(node.to_expression(), false);
            if let Some(expr) = expr {
              // TODO: This is not the minimal representation, to fix this we need two expression nodes.
              (
                true,
                self.ast_builder.property_key_expression(build_effect!(
                  self.ast_builder,
                  span,
                  Some(expr);
                  self.ast_builder.expression_string_literal(SPAN, s)
                )),
              )
            } else {
              // FIXME: Only valid identifier names are allowed
              (false, self.ast_builder.property_key_identifier_name(span, s))
            }
          })
        } else {
          let expr = self.transform_expression(node.to_expression(), need_val);
          expr.map(|e| (true, self.ast_builder.property_key_expression(e)))
        }
      }
    }
  }
}
