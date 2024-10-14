use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  build_effect_from_arr,
  consumable::box_consumable,
  entity::{Entity, EntityDepNode},
  transformer::Transformer,
};
use oxc::{
  ast::{
    ast::{Expression, TaggedTemplateExpression, TemplateLiteral},
    AstKind, NONE,
  },
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub fn exec_tagged_template_expression(
    &mut self,
    node: &'a TaggedTemplateExpression<'a>,
  ) -> Entity<'a> {
    if let Some((indeterminate, tag, this)) = self.exec_callee(&node.tag) {
      if indeterminate {
        self.push_indeterminate_cf_scope();
      }

      let mut arguments = vec![(false, self.factory.unknown)];

      for expr in &node.quasi.expressions {
        let value = self.exec_expression(expr);
        let dep: EntityDepNode = (AstType2::ExpressionInTaggedTemplate, expr).into();
        arguments.push((false, self.factory.computed(value, dep)));
      }

      let value = tag.call(
        self,
        box_consumable(AstKind::TaggedTemplateExpression(node)),
        this,
        self.factory.arguments(arguments),
      );

      if indeterminate {
        self.pop_cf_scope();
        self.factory.union(vec![value, self.factory.undefined])
      } else {
        value
      }
    } else {
      self.factory.undefined
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_tagged_template_expression(
    &self,
    node: &'a TaggedTemplateExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let TaggedTemplateExpression { span, tag, quasi, .. } = node;

    let need_call = need_val || self.is_referred(AstKind::TaggedTemplateExpression(node));

    if need_call {
      let tag = self.transform_callee(tag, true).unwrap();

      Some(self.ast_builder.expression_tagged_template(
        *span,
        tag,
        self.transform_quasi(quasi),
        NONE,
      ))
    } else {
      build_effect_from_arr!(
        &self.ast_builder,
        *span,
        vec![self.transform_callee(tag, false)],
        quasi.expressions.iter().map(|x| self.transform_expression(x, false))
      )
    }
  }

  fn transform_quasi(&self, node: &'a TemplateLiteral<'a>) -> TemplateLiteral<'a> {
    let TemplateLiteral { span, quasis, expressions } = node;

    let mut transformed_expressions = self.ast_builder.vec();
    for expr in expressions {
      let expr_span = expr.span();
      let referred = self.is_referred((AstType2::ExpressionInTaggedTemplate, expr));
      transformed_expressions.push(
        self
          .transform_expression(expr, referred)
          .unwrap_or_else(|| self.build_unused_expression(expr_span)),
      );
    }

    self.ast_builder.template_literal(*span, self.clone_node(quasis), transformed_expressions)
  }
}
