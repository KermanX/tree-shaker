use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  build_effect,
  entity::{Entity, EntityDepNode, UnionEntity},
  scope::{conditional::ConditionalData, CfScopeKind},
  transformer::Transformer,
};
use oxc::ast::{
  ast::{Expression, LogicalExpression, LogicalOperator},
  AstKind,
};

const AST_TYPE: AstType2 = AstType2::LogicalExpression;

#[derive(Debug, Default)]
pub struct Data<'a> {
  need_left_val: bool,
  need_right: bool,
  conditional: ConditionalData<'a>,
}

impl<'a> Analyzer<'a> {
  pub fn exec_logical_expression(&mut self, node: &'a LogicalExpression<'a>) -> Entity<'a> {
    let left = self.exec_expression(&node.left);

    let (need_left_val, need_right) = match &node.operator {
      LogicalOperator::And => match left.test_truthy() {
        Some(true) => (false, true),
        Some(false) => (true, false),
        None => (true, true),
      },
      LogicalOperator::Or => match left.test_truthy() {
        Some(true) => (true, false),
        Some(false) => (false, true),
        None => (true, true),
      },
      LogicalOperator::Coalesce => match left.test_nullish() {
        Some(true) => (false, true),
        Some(false) => (true, false),
        None => (true, true),
      },
    };

    let data = self.load_data::<Data>(AST_TYPE, node);

    data.need_left_val |= need_left_val;
    data.need_right |= need_right;

    let historical_indeterminate = data.need_left_val && data.need_right;
    let current_indeterminate = need_left_val && need_right;

    self.push_conditional_cf_scope(
      &mut data.conditional,
      CfScopeKind::LogicalExpression,
      left.clone(),
      historical_indeterminate,
      current_indeterminate,
    );
    self.push_cf_scope_for_dep(EntityDepNode::from(AstKind::LogicalExpression(node)));

    let value = match (need_left_val, need_right) {
      (false, true) => self.exec_expression(&node.right),
      (true, false) => left,
      (true, true) => UnionEntity::new(vec![left, self.exec_expression(&node.right)]),
      (false, false) => unreachable!(),
    };

    self.pop_cf_scope();
    self.pop_cf_scope();

    value
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_logical_expression(
    &self,
    node: &'a LogicalExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);
    let need_left_val =
      data.need_left_val && (need_val || self.is_referred(AstKind::LogicalExpression(node)));

    let LogicalExpression { span, left, operator, right, .. } = node;

    let left = self.transform_expression(left, need_left_val);
    let right = data.need_right.then(|| self.transform_expression(right, need_val)).flatten();

    match (left, right) {
      (Some(left), Some(right)) => {
        if need_left_val {
          Some(self.ast_builder.expression_logical(*span, left, *operator, right))
        } else {
          Some(build_effect!(self.ast_builder, *span, Some(left); right))
        }
      }
      (Some(left), None) => Some(left),
      (None, Some(right)) => Some(right),
      (None, None) => None,
    }
  }
}
