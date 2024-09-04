use crate::ast::AstType2;
use crate::build_effect;
use crate::entity::entity::Entity;
use crate::entity::union::UnionEntity;
use crate::{analyzer::Analyzer, Transformer};
use oxc::ast::ast::{Expression, LogicalExpression, LogicalOperator};
use std::rc::Rc;

const AST_TYPE: AstType2 = AstType2::LogicalExpression;

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_left_val: bool,
  need_right: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_logical_expression(&mut self, node: &'a LogicalExpression<'a>) -> Entity<'a> {
    let left = self.exec_expression(&node.left);

    let exec_right = |analyzer: &mut Analyzer<'a>| analyzer.exec_expression(&node.right);

    let exec_unknown = |analyzer: &mut Analyzer<'a>| {
      analyzer.push_cf_scope(None, false);
      let right = analyzer.exec_expression(&node.right);
      analyzer.pop_cf_scope();
      (Rc::new(UnionEntity(vec![left.clone(), right])) as Entity<'a>, true, true)
    };

    let (value, need_left_val, need_right) = match &node.operator {
      LogicalOperator::And => match left.test_truthy() {
        Some(true) => (exec_right(self), false, true),
        Some(false) => (left, true, false),
        None => exec_unknown(self),
      },
      LogicalOperator::Or => match left.test_truthy() {
        Some(true) => (left, true, false),
        Some(false) => (exec_right(self), false, true),
        None => exec_unknown(self),
      },
      LogicalOperator::Coalesce => match left.test_nullish() {
        Some(true) => (exec_right(self), false, true),
        Some(false) => (left, true, false),
        None => exec_unknown(self),
      },
    };

    let data = self.load_data::<Data>(AST_TYPE, node);

    data.need_left_val |= need_left_val;
    data.need_right |= need_right;

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

    let LogicalExpression { span, left, operator, right, .. } = node;

    let left = self.transform_expression(left, need_val && data.need_left_val);
    let right = data.need_right.then(|| self.transform_expression(right, need_val)).flatten();

    match (left, right) {
      (Some(left), Some(right)) => {
        if need_val && data.need_left_val {
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
