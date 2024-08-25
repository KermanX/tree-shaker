use crate::ast::AstType2;
use crate::build_effect;
use crate::entity::collector::LiteralCollector;
use crate::entity::entity::Entity;
use crate::entity::literal::LiteralEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::span::SPAN;
use oxc::{ast::ast::PropertyKey, span::GetSpan};
use std::rc::Rc;

const AST_TYPE: AstType2 = AstType2::PropertyKey;

#[derive(Debug, Default)]
pub struct Data<'a> {
  collector: LiteralCollector<'a>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_property_key(&mut self, node: &'a PropertyKey) -> Entity<'a> {
    let value = match node {
      PropertyKey::StaticIdentifier(node) => Rc::new(LiteralEntity::String(node.name.as_str())),
      PropertyKey::PrivateIdentifier(node) => todo!(),
      node => {
        let node = node.to_expression();
        self.exec_expression(node)
      }
    };

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.collector.collect(&value);

    value
  }
}

impl<'a> Transformer<'a> {
  /// Returns (computed, node)
  pub(crate) fn transform_property_key(
    &mut self,
    node: PropertyKey<'a>,
    need_val: bool,
  ) -> Option<(bool, PropertyKey<'a>)> {
    match node {
      // Reuse the node
      PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => {
        need_val.then_some((false, node))
      }
      _ => {
        let data = self.get_data::<Data>(AST_TYPE, &node);
        if let Some(literal) = data.collector.collected() {
          if let LiteralEntity::String(s) = literal {
            let span = node.span();
            let expr = self.transform_expression(TryFrom::try_from(node).unwrap(), false);
            if let Some(expr) = expr {
              // TODO: This is not the minimal representation
              Some((
                true,
                self.ast_builder.property_key_expression(build_effect!(
                  self.ast_builder,
                  span,
                  Some(expr);
                  self.ast_builder.expression_string_literal(SPAN, s)
                )),
              ))
            } else {
              Some((false, self.ast_builder.property_key_identifier_name(span, s)))
            }
          } else {
            unreachable!()
          }
        } else {
          let expr = self.transform_expression(node.try_into().unwrap(), need_val);
          expr.map(|e| (true, self.ast_builder.property_key_expression(e)))
        }
      }
    }
  }
}
