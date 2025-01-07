use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::{AssignmentExpression, AssignmentOperator};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_assignment_expression(&mut self, node: &'a AssignmentExpression<'a>) -> H::Entity {
    if node.operator == AssignmentOperator::Assign {
      let rhs = self.exec_expression(&node.right);
      self.exec_assignment_target_write(&node.left, rhs, None);
      rhs
    } else if node.operator.is_logical() {
      let (left, cache) = self.exec_assignment_target_read(&node.left);

      let (maybe_left, maybe_right) = match &node.operator {
        AssignmentOperator::LogicalAnd => match left.test_truthy() {
          Some(true) => (false, true),
          Some(false) => (true, false),
          None => (true, true),
        },
        AssignmentOperator::LogicalOr => match left.test_truthy() {
          Some(true) => (true, false),
          Some(false) => (false, true),
          None => (true, true),
        },
        AssignmentOperator::LogicalNullish => match left.test_nullish() {
          Some(true) => (false, true),
          Some(false) => (true, false),
          None => (true, true),
        },
        _ => unreachable!(),
      };

      let forward_left = |analyzer: &mut Analyzer<'a>| {
        analyzer.forward_logical_left_val(
          AstKind2::LogicalAssignmentExpressionLeft(node),
          left,
          maybe_left,
          maybe_right,
        )
      };
      let exec_right = |analyzer: &mut Analyzer<'a>| {
        let conditional_dep = analyzer.push_logical_right_cf_scope(
          AstKind2::LogicalAssignmentExpressionLeft(node),
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
          self.factory.logical_result(left, right, to_logical_operator(node.operator))
        }
        (false, false) => {
          unreachable!("Logical assignment expression should have at least one side")
        }
      };

      if maybe_right {
        self.exec_assignment_target_write(&node.left, value, cache);
      }

      value
    } else {
      let (lhs, cache) = self.exec_assignment_target_read(&node.left);
      let rhs = self.exec_expression(&node.right);
      let value = self.entity_op.binary_op(self, to_binary_operator(node.operator), lhs, rhs);
      self.exec_assignment_target_write(&node.left, value, cache);
      value
    }
  }
}
