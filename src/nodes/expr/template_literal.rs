use crate::{analyzer::Analyzer, build_effect_from_arr, entity::Entity, transformer::Transformer};
use oxc::{
  ast::ast::{Expression, TemplateElementValue, TemplateLiteral},
  span::{GetSpan, SPAN},
};
use std::mem;

impl<'a> Analyzer<'a> {
  pub fn exec_template_literal(&mut self, node: &'a TemplateLiteral<'a>) -> Entity<'a> {
    let mut result = self.factory.new_string(node.quasi().unwrap().as_str());
    for (index, expression) in node.expressions.iter().enumerate() {
      let expression = self.exec_expression(expression);
      let quasi =
        self.factory.new_string(node.quasis.get(index + 1).unwrap().value.cooked.as_ref().unwrap());
      result = self.entity_op.add(self, result, expression);
      result = self.entity_op.add(self, result, quasi);
    }
    result
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_template_literal(
    &self,
    node: &'a TemplateLiteral<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let TemplateLiteral { span, expressions, quasis, .. } = node;
    if need_val {
      let mut quasis_iter = quasis.into_iter();
      let mut transformed_exprs = self.ast_builder.vec();
      let mut transformed_quasis = self.ast_builder.vec();
      let mut pending_effects = vec![];
      transformed_quasis
        .push(quasis_iter.next().unwrap().value.cooked.as_ref().unwrap().to_string());
      let exprs_len = expressions.len();
      for (index, expr) in expressions.into_iter().enumerate() {
        let is_last = index == exprs_len - 1;
        let expr_span = expr.span();
        let quasi_str = quasis_iter.next().unwrap().value.cooked.as_ref().unwrap().to_string();
        if let Some(literal) = self.get_expression_collected_literal(expr) {
          if let Some(effect) = self.transform_expression(expr, false) {
            pending_effects.push(Some(effect));
          }
          if !pending_effects.is_empty() && is_last {
            transformed_exprs.push(build_effect_from_arr!(
              &self.ast_builder,
              expr_span,
              mem::take(&mut pending_effects);
              literal.build_expr(&self.ast_builder, SPAN)
            ));
            transformed_quasis.push(quasi_str);
          } else {
            let last_quasi = transformed_quasis.pop().unwrap();
            let expr_str = literal.to_string();
            transformed_quasis.push(format!("{}{}{}", last_quasi, expr_str, quasi_str));
          }
        } else {
          let expr = self.transform_expression(expr, true).unwrap();
          transformed_exprs.push(build_effect_from_arr!(
            &self.ast_builder,
            expr_span,
            mem::take(&mut pending_effects);
            expr
          ));
          transformed_quasis.push(quasi_str);
        }
      }
      if transformed_exprs.is_empty() {
        Some(build_effect_from_arr!(
          &self.ast_builder,
          *span,
          pending_effects;
          self.ast_builder.expression_string_literal(*span, transformed_quasis.first().unwrap().clone())
        ))
      } else {
        assert!(pending_effects.is_empty());
        let mut quasis = self.ast_builder.vec();
        let quasis_len = transformed_quasis.len();
        for (index, quasi) in transformed_quasis.into_iter().enumerate() {
          let str = self.allocator.alloc(quasi);
          quasis.push(self.ast_builder.template_element(
            *span,
            index == quasis_len - 1,
            TemplateElementValue {
              // FIXME: escape
              raw: str.as_str().into(),
              cooked: Some(str.as_str().into()),
            },
          ));
        }
        Some(self.ast_builder.expression_template_literal(*span, quasis, transformed_exprs))
      }
    } else {
      build_effect_from_arr!(
        &self.ast_builder,
        *span,
        expressions.into_iter().map(|x| self.transform_expression(x, false))
      )
    }
  }
}
