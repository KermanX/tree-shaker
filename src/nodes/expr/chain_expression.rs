use crate::{analyzer::Analyzer, entity::Entity, transformer::Transformer};
use oxc::ast::{
  ast::{ChainElement, ChainExpression, Expression, MemberExpression},
  match_member_expression,
};

impl<'a> Analyzer<'a> {
  pub fn exec_chain_expression(&mut self, node: &'a ChainExpression<'a>) -> Entity<'a> {
    match &node.expression {
      ChainElement::CallExpression(node) => self.exec_call_expression_in_chain(node, None).1,
      node => self.exec_member_expression_read_in_chain(node.to_member_expression(), false).1,
    }
  }

  pub fn exec_expression_in_chain(
    &mut self,
    node: &'a Expression<'a>,
  ) -> (Option<bool>, Entity<'a>) {
    match node {
      match_member_expression!(Expression) => {
        let (short_circuit, value, _cache) =
          self.exec_member_expression_read_in_chain(node.to_member_expression(), false);
        (short_circuit, value)
      }
      Expression::CallExpression(node) => self.exec_call_expression_in_chain(node, None),
      _ => (Some(false), self.exec_expression(node)),
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_chain_expression(
    &self,
    node: &'a ChainExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let ChainExpression { span, expression } = node;

    let expression = match expression {
      ChainElement::CallExpression(node) => self.transform_call_expression(node, need_val),
      node => self.transform_member_expression_read(node.to_member_expression(), need_val),
    };

    // FIXME: is this correct?
    expression.map(|expression| match expression {
      Expression::CallExpression(node) => {
        self.ast_builder.expression_chain(*span, ChainElement::CallExpression(node))
      }
      match_member_expression!(Expression) => self.ast_builder.expression_chain(
        *span,
        ChainElement::from(MemberExpression::try_from(expression).unwrap()),
      ),
      _ => expression,
    })
  }
}
