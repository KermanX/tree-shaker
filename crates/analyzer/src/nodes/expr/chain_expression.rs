use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::{
  ast::{ChainElement, ChainExpression, Expression},
  match_member_expression,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_chain_expression(&mut self, node: &'a ChainExpression<'a>) -> H::Entity {
    match &node.expression {
      ChainElement::CallExpression(node) => {
        let result = self.exec_call_expression_in_chain(node);
        match result {
          Ok((scope_count, value, undefined)) => {
            let exec_dep = self.pop_multiple_cf_scopes(scope_count);
            self.factory.optional_union(self.factory.optional_computed(value, exec_dep), undefined)
          }
          Err(value) => value,
        }
      }
      node => {
        let result = self.exec_member_expression_read_in_chain(node.to_member_expression(), false);
        match result {
          Ok((scope_count, value, undefined, _)) => {
            let exec_dep = self.pop_multiple_cf_scopes(scope_count);
            self.factory.optional_union(self.factory.optional_computed(value, exec_dep), undefined)
          }
          Err(value) => value,
        }
      }
    }
  }

  pub fn exec_expression_in_chain(
    &mut self,
    node: &'a Expression<'a>,
  ) -> Result<(usize, H::Entity, Option<H::Entity>), H::Entity> {
    match node {
      match_member_expression!(Expression) => self
        .exec_member_expression_read_in_chain(node.to_member_expression(), false)
        .map(|(scope_count, value, undefined, _)| (scope_count, value, undefined)),
      Expression::CallExpression(node) => self.exec_call_expression_in_chain(node),
      _ => Ok((0, self.exec_expression(node), None)),
    }
  }
}

