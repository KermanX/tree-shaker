use crate::{
  analyzer::Analyzer,
  entity::{ArrayEntity, Entity},
  transformer::Transformer,
  utils::ast::AstKind2,
};
use ecma_analyzer::ArrayExpressionAnalyzer;
use oxc::{ast::ast::{ArrayExpression, ArrayExpressionElement, Expression, SpreadElement}, span::GetSpan};

impl<'a> ArrayExpressionAnalyzer<'a> for Analyzer<'a> {
  type Context = &'a mut ArrayEntity<'a>;

  fn before_array_expression(&mut self, _node: &'a ArrayExpression<'a>) -> Self::Context {
    self.new_empty_array()
  }

  fn init_element(
    &mut self,
    node: &'a ArrayExpressionElement<'a>,
    context: &mut Self::Context,
    value: Entity<'a>,
  ) {
    let dep = self.consumable(AstKind2::ArrayExpressionElement(node));
    let value = self.factory.computed(value, dep);
    context.push_element(value);
  }

  fn init_spread(
    &mut self,
    node: &'a SpreadElement<'a>,
    context: &mut Self::Context,
    value: Entity<'a>,
  ) {
    let dep = self.consumable(AstKind2::SpreadElement(node));
    if let Some(union) = value.iterate_result_union(self, dep) {
      context.init_rest(union);
    }
  }

  fn after_array_expression(&mut self, context: Self::Context) -> Entity<'a> {
    context
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_array_expression(
    &self,
    node: &'a ArrayExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let ArrayExpression { span, elements, trailing_comma } = node;

    let mut transformed_elements = self.ast_builder.vec();

    for element in elements {
      let span = element.span();
      match element {
        ArrayExpressionElement::SpreadElement(node) => {
          if let Some(element) = self.transform_spread_element(node, need_val) {
            transformed_elements.push(element);
          }
        }
        ArrayExpressionElement::Elision(_) => {
          if need_val {
            transformed_elements.push(self.ast_builder.array_expression_element_elision(span));
          }
        }
        _ => {
          let referred = self.is_referred(AstKind2::ArrayExpressionElement(element));
          let element = self.transform_expression(element.to_expression(), need_val && referred);
          if let Some(inner) = element {
            transformed_elements.push(inner.into());
          } else if need_val {
            transformed_elements.push(self.ast_builder.array_expression_element_elision(span));
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
                self.ast_builder.vec1(ArrayExpressionElement::SpreadElement(inner)),
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
