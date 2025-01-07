use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  allocator,
  ast::ast::{Expression, JSXExpression, JSXExpressionContainer},
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_expression_container_as_attribute_value(
    &mut self,
    node: &'a JSXExpressionContainer<'a>,
  ) -> H::Entity {
    match &node.expression {
      JSXExpression::EmptyExpression(_node) => self.factory.r#true,
      node => self.exec_expression(node.to_expression()),
    }
  }

  pub fn exec_jsx_expression_container_as_jsx_child(
    &mut self,
    node: &'a JSXExpressionContainer<'a>,
  ) -> H::Entity {
    let value = match &node.expression {
      JSXExpression::EmptyExpression(_node) => self.factory.string(""),
      node => self.exec_expression(node.to_expression()).get_to_jsx_child(self),
    };

    data.collector.collect(self, value)
  }
}
