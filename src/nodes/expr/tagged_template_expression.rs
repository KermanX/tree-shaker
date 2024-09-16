use crate::{
  analyzer::Analyzer,
  build_effect_from_arr,
  entity::{arguments::ArgumentsEntity, entity::Entity, unknown::UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::{
  ast::{Expression, TSTypeParameterInstantiation, TaggedTemplateExpression, TemplateLiteral},
  AstKind,
};

impl<'a> Analyzer<'a> {
  pub fn exec_tagged_template_expression(
    &mut self,
    node: &'a TaggedTemplateExpression<'a>,
  ) -> Entity<'a> {
    let tag = self.exec_expression(&node.tag);

    let mut arguments = vec![(false, UnknownEntity::new_unknown())];

    for expr in &node.quasi.expressions {
      arguments.push((false, self.exec_expression(expr)));
    }

    // TODO: this
    tag.call(
      self,
      AstKind::TaggedTemplateExpression(node),
      &UnknownEntity::new_unknown(),
      &ArgumentsEntity::new(arguments),
    )
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
      let tag = self.transform_expression(tag, true).unwrap();

      Some(self.ast_builder.expression_tagged_template(
        *span,
        tag,
        self.transform_quasi(quasi),
        None::<TSTypeParameterInstantiation>,
      ))
    } else {
      build_effect_from_arr!(
        &self.ast_builder,
        *span,
        vec![self.transform_expression(tag, false)],
        quasi.expressions.iter().map(|x| self.transform_expression(x, false))
      )
    }
  }

  fn transform_quasi(&self, node: &'a TemplateLiteral<'a>) -> TemplateLiteral<'a> {
    let TemplateLiteral { span, quasis, expressions } = node;

    let mut transformed_expressions = self.ast_builder.vec();
    for expression in expressions {
      transformed_expressions.push(self.transform_expression(expression, true).unwrap());
    }

    self.ast_builder.template_literal(*span, self.clone_node(quasis), transformed_expressions)
  }
}
