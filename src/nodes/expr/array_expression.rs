use crate::{
  analyzer::Analyzer,
  entity::{Entity, LiteralEntity, UnionEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{ArrayExpression, ArrayExpressionElement, Expression, SpreadElement};

impl<'a> Analyzer<'a> {
  pub fn exec_array_expression(&mut self, node: &'a ArrayExpression<'a>) -> Entity<'a> {
    let array = self.new_empty_array();

    let mut rest = vec![];

    for element in &node.elements {
      match element {
        ArrayExpressionElement::SpreadElement(node) => {
          if let Some(spreaded) = self.exec_spread_element(node) {
            rest.push(spreaded);
          }
        }
        ArrayExpressionElement::Elision(_node) => {
          if rest.is_empty() {
            array.push_element(LiteralEntity::new_undefined());
          } else {
            rest.push(LiteralEntity::new_undefined());
          }
        }
        _ => {
          let element = self.exec_expression(element.to_expression());
          if rest.is_empty() {
            array.push_element(element);
          } else {
            rest.push(element);
          }
        }
      }
    }

    if !rest.is_empty() {
      array.init_rest(UnionEntity::new(rest));
    }

    Entity::new(array)
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_array_expression(
    &self,
    node: &'a ArrayExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let ArrayExpression { span, elements, trailing_comma, .. } = node;

    let mut transformed_elements = self.ast_builder.vec();

    for element in elements {
      match element {
        ArrayExpressionElement::SpreadElement(node) => {
          if let Some(element) = self.transform_spread_element(node, need_val) {
            transformed_elements.push(element);
          }
        }
        ArrayExpressionElement::Elision(node) => {
          if need_val {
            transformed_elements.push(self.ast_builder.array_expression_element_elision(node.span));
          }
        }
        _ => {
          let element = self.transform_expression(element.to_expression(), need_val);
          if let Some(inner) = element {
            transformed_elements.push(self.ast_builder.array_expression_element_expression(inner));
          }
        }
      }
    }

    if !need_val {
      if transformed_elements.is_empty() {
        return None;
      }
      if transformed_elements.len() == 1 {
        return Some(match transformed_elements.pop().unwrap() {
          ArrayExpressionElement::SpreadElement(inner) => {
            if self.config.iterate_side_effects {
              self.ast_builder.expression_array(
                *span,
                self
                  .ast_builder
                  .vec1(self.ast_builder.array_expression_element_from_spread_element(inner)),
                None,
              )
            } else {
              let SpreadElement { argument, .. } = inner.unbox();
              argument
            }
          }
          node => node.try_into().unwrap(),
        });
      }
    }

    Some(self.ast_builder.expression_array(*span, transformed_elements, *trailing_comma))
  }
}
