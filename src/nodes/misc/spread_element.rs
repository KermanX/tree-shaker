use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::ast::{
  ast::{ArrayExpressionElement, SpreadElement},
  AstKind,
};

impl<'a> Analyzer<'a> {
  pub fn exec_spread_element(&mut self, node: &'a SpreadElement<'a>) -> Option<Entity<'a>> {
    let argument = self.exec_expression(&node.argument);
    argument.iterate_result_union(self, AstKind::SpreadElement(node))
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_spread_element(
    &self,
    node: &'a SpreadElement<'a>,
    need_val: bool,
  ) -> Option<ArrayExpressionElement<'a>> {
    let SpreadElement { span, argument } = node;

    let need_spread = need_val || self.is_referred(AstKind::SpreadElement(node));

    let argument = self.transform_expression(argument, need_spread);

    if let Some(argument) = argument {
      Some(if need_spread {
        self.ast_builder.array_expression_element_spread_element(*span, argument)
      } else {
        self.ast_builder.array_expression_element_expression(argument)
      })
    } else {
      None
    }
  }
}
