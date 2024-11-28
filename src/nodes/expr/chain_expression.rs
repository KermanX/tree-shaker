use crate::{analyzer::Analyzer, entity::Entity, transformer::Transformer};
use oxc::ast::{
  ast::{ChainElement, ChainExpression, Expression},
  match_member_expression,
};

impl<'a> Analyzer<'a> {
  pub fn exec_chain_expression(&mut self, node: &'a ChainExpression<'a>) -> Entity<'a> {
    match &node.expression {
      ChainElement::CallExpression(node) => {
        let result = self.exec_call_expression_in_chain(node, None);
        match result {
          Ok((scope_count, value, undefined)) => {
            self.pop_multiple_cf_scopes(scope_count);
            self.factory.optional_union(value, undefined)
          }
          Err(value) => value,
        }
      }
      node => {
        let result = self.exec_member_expression_read_in_chain(node.to_member_expression(), false);
        match result {
          Ok((scope_count, value, undefined, _)) => {
            self.pop_multiple_cf_scopes(scope_count);
            self.factory.optional_union(value, undefined)
          }
          Err(value) => value,
        }
      }
    }
  }

  pub fn exec_expression_in_chain(
    &mut self,
    node: &'a Expression<'a>,
  ) -> Result<(usize, Entity<'a>, Option<Entity<'a>>), Entity<'a>> {
    match node {
      match_member_expression!(Expression) => self
        .exec_member_expression_read_in_chain(node.to_member_expression(), false)
        .map(|(scope_count, value, undefined, _)| (scope_count, value, undefined)),
      Expression::CallExpression(node) => self.exec_call_expression_in_chain(node, None),
      _ => Ok((0, self.exec_expression(node), None)),
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_chain_expression(
    &self,
    node: &'a ChainExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let ChainExpression { expression, .. } = node;

    match expression {
      ChainElement::CallExpression(node) => {
        self.transform_call_expression_in_chain(node, need_val, None)
      }
      node => {
        self.transform_member_expression_read_in_chain(node.to_member_expression(), need_val, None)
      }
    }
  }

  pub fn transform_expression_in_chain(
    &self,
    node: &'a Expression<'a>,
    need_val: bool,
    parent_effects: Option<Expression<'a>>,
  ) -> Option<Expression<'a>> {
    match node {
      match_member_expression!(Expression) => self.transform_member_expression_read_in_chain(
        node.to_member_expression(),
        need_val,
        parent_effects,
      ),
      Expression::CallExpression(node) => {
        self.transform_call_expression_in_chain(node, need_val, parent_effects)
      }
      expression => self.transform_expression(expression, need_val),
    }
  }
}
