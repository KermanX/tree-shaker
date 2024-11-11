use crate::{analyzer::Analyzer, entity::Entity, transformer::Transformer};
use oxc::ast::ast::{Expression, JSXExpression, JSXExpressionContainer};

impl<'a> Analyzer<'a> {
  pub fn exec_jsx_expression_container(
    &mut self,
    node: &'a JSXExpressionContainer<'a>,
  ) -> Entity<'a> {
    match &node.expression {
      JSXExpression::EmptyExpression(_node) => self.factory.r#true,
      node => self.exec_expression(node.to_expression()),
    }
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
  ) -> JSXExpressionContainer<'a> {
    let JSXExpressionContainer { span, expression } = node;

    self.ast_builder.jsx_expression_container(
      *span,
      match expression {
        JSXExpression::EmptyExpression(node) => {
          self.ast_builder.jsx_expression_jsx_empty_expression(node.span)
        }
        node => self.ast_builder.jsx_expression_expression(
          self.transform_expression(node.to_expression(), true).unwrap(),
        ),
      },
    )
  }
}
