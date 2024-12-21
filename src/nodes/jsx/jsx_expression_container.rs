use crate::{
  analyzer::Analyzer,
  ast::AstKind2,
  build_effect,
  entity::{Entity, LiteralCollector, LiteralEntity},
  transformer::Transformer,
};
use oxc::{
  allocator,
  ast::ast::{Expression, JSXExpression, JSXExpressionContainer},
  span::GetSpan,
};

#[derive(Default)]
struct AsJsxChildData<'a> {
  collector: LiteralCollector<'a>,
}

impl<'a> Analyzer<'a> {
  pub fn exec_jsx_expression_container_as_attribute_value(
    &mut self,
    node: &'a JSXExpressionContainer<'a>,
  ) -> Entity<'a> {
    match &node.expression {
      JSXExpression::EmptyExpression(_node) => self.factory.r#true,
      node => self.exec_expression(node.to_expression()),
    }
  }

  pub fn exec_jsx_expression_container_as_jsx_child(
    &mut self,
    node: &'a JSXExpressionContainer<'a>,
  ) -> Entity<'a> {
    let data = self.load_data::<AsJsxChildData>(AstKind2::JsxExpressionContainer(node));

    let value = match &node.expression {
      JSXExpression::EmptyExpression(_node) => self.factory.string(""),
      node => self.exec_expression(node.to_expression()).get_to_jsx_child(self),
    };

    data.collector.collect(self, value)
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_jsx_expression_container_effect_only(
    &self,
    node: &'a JSXExpressionContainer<'a>,
  ) -> Option<Expression<'a>> {
    match &node.expression {
      JSXExpression::EmptyExpression(_node) => None,
      node => self.transform_expression(node.to_expression(), false),
    }
  }

  pub fn transform_jsx_expression_container_need_val(
    &self,
    node: &'a JSXExpressionContainer<'a>,
  ) -> allocator::Box<'a, JSXExpressionContainer<'a>> {
    let data = self.get_data::<AsJsxChildData>(AstKind2::JsxExpressionContainer(node));

    let JSXExpressionContainer { span, expression } = node;

    self.ast_builder.alloc_jsx_expression_container(
      *span,
      if let Some(literal) = data.collector.build_expr(&self.ast_builder, *span) {
        let effect = self.transform_jsx_expression_container_effect_only(node);
        if effect.is_none()
          && matches!(data.collector.collected().unwrap(), LiteralEntity::String("", _))
        {
          self.ast_builder.jsx_expression_jsx_empty_expression(expression.span())
        } else {
          JSXExpression::from(build_effect!(self.ast_builder, expression.span(), effect; literal))
        }
      } else {
        match expression {
          JSXExpression::EmptyExpression(node) => {
            self.ast_builder.jsx_expression_jsx_empty_expression(node.span)
          }
          node => {
            JSXExpression::from(self.transform_expression(node.to_expression(), true).unwrap())
          }
        }
      },
    )
  }
}
