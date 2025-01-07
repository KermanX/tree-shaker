use crate::{host::Host, 
  analyzer::Analyzer,  
};
use oxc::ast::ast::{Expression, LogicalExpression, LogicalOperator};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_logical_expression(&mut self, node: &'a LogicalExpression<'a>) -> H::Entity {
    let left = self.exec_expression(&node.left);

    let (maybe_left, maybe_right) = match &node.operator {
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

    let forward_left = |analyzer: &mut Analyzer<'a>| {
      analyzer.forward_logical_left_val(
        AstKind2::LogicalExpressionLeft(node),
        left,
        maybe_left,
        maybe_right,
      )
    };

    let exec_right = |analyzer: &mut Analyzer<'a>| {
      let conditional_dep = analyzer.push_logical_right_cf_scope(
        AstKind2::LogicalExpressionLeft(node),
        left,
        maybe_left,
        maybe_right,
      );

      let val = analyzer.factory.computed(analyzer.exec_expression(&node.right), conditional_dep);

      analyzer.pop_cf_scope();

      val
    };

    let value = match (maybe_left, maybe_right) {
      (false, true) => exec_right(self),
      (true, false) => forward_left(self),
      (true, true) => {
        let left = forward_left(self);
        let right = exec_right(self);
        self.factory.logical_result(left, right, node.operator)
      }
      (false, false) => unreachable!("Logical expression should have at least one possible branch"),
    };

    value
  }
}

