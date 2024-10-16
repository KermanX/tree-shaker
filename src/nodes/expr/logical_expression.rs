use crate::{
  analyzer::Analyzer, ast::AstType2, build_effect, entity::Entity, transformer::Transformer,
};
use oxc::ast::ast::{Expression, LogicalExpression, LogicalOperator};

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

    let conditional_dep = self.push_logical_right_cf_cope(
      (AstType2::LogicalExpressionLeft, &node.left),
      left.clone(),
      need_left_val,
      need_right,
    );

    let exec_right = |analyzer: &mut Analyzer<'a>| {
      let val = analyzer.exec_expression(&node.right);
      analyzer.factory.computed(val, conditional_dep)
    };

    let value = match (need_left_val, need_right) {
      (false, true) => exec_right(self),
      (true, false) => left,
      (true, true) => {
        let right = exec_right(self);
        self.factory.union(vec![left, right])
      }
      (false, false) => unreachable!(),
    };

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
    let LogicalExpression { span, left, operator, right, .. } = node;

    let (need_left_test_val, need_left_val, need_right) =
      self.get_conditional_result((AstType2::LogicalExpressionLeft, &node.left));

    let need_left_val = (need_val && need_left_val) || need_left_test_val;
    let left = self.transform_expression(left, need_left_val);
    let right = need_right.then(|| self.transform_expression(right, need_val)).flatten();

    if need_left_test_val {
      let left = left.unwrap();
      if let Some(right) = right {
        Some(self.ast_builder.expression_logical(*span, left, *operator, right))
      } else {
        Some(left)
      }
    } else {
      build_effect!(self.ast_builder, *span, left, right)
    }
  }
}
