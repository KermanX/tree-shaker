use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{entity::Entity, union::UnionEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{
  AssignmentExpression, AssignmentOperator, BinaryOperator, Expression, LogicalOperator,
};

const AST_TYPE: AstType2 = AstType2::AssignmentExpression;

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_left_val: bool,
  need_right: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_assignment_expression(&mut self, node: &'a AssignmentExpression<'a>) -> Entity<'a> {
    if node.operator == AssignmentOperator::Assign {
      let rhs = self.exec_expression(&node.right);
      self.exec_assignment_target_write(&node.left, rhs.clone(), None);
      rhs
    } else if node.operator.is_logical() {
      let (left, cache) = self.exec_assignment_target_read(&node.left);

      let exec_right = |analyzer: &mut Analyzer<'a>| analyzer.exec_expression(&node.right);

      let exec_unknown = |analyzer: &mut Analyzer<'a>| {
        analyzer.push_cf_scope_normal(None);
        let right = analyzer.exec_expression(&node.right);
        analyzer.pop_cf_scope();
        (UnionEntity::new(vec![left.clone(), right]), true, true)
      };

      let (value, need_left_val, need_right) = match &node.operator {
        AssignmentOperator::LogicalAnd => match left.test_truthy() {
          Some(true) => (exec_right(self), false, true),
          Some(false) => (left, true, false),
          None => exec_unknown(self),
        },
        AssignmentOperator::LogicalOr => match left.test_truthy() {
          Some(true) => (left, true, false),
          Some(false) => (exec_right(self), false, true),
          None => exec_unknown(self),
        },
        AssignmentOperator::LogicalNullish => match left.test_nullish() {
          Some(true) => (exec_right(self), false, true),
          Some(false) => (left, true, false),
          None => exec_unknown(self),
        },
        _ => unreachable!(),
      };

      let data = self.load_data::<Data>(AST_TYPE, node);

      data.need_left_val |= need_left_val;
      data.need_right |= need_right;

      if need_right {
        self.exec_assignment_target_write(&node.left, value.clone(), cache);
      }

      value
    } else {
      let (lhs, cache) = self.exec_assignment_target_read(&node.left);
      let rhs = self.exec_expression(&node.right);
      let value = self.entity_op.binary_op(to_binary_operator(node.operator), &lhs, &rhs);
      self.exec_assignment_target_write(&node.left, value.clone(), cache);
      value
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_assignment_expression(
    &self,
    node: &'a AssignmentExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let AssignmentExpression { span, operator, left, right } = node;

    let (left_is_empty, transformed_left) =
      self.transform_assignment_target_write(left, false, false);
    let transformed_right = self.transform_expression(right, need_val || !left_is_empty);

    match (transformed_left, transformed_right) {
      (Some(left), Some(right)) => {
        Some(self.ast_builder.expression_assignment(*span, *operator, left, right))
      }
      (None, Some(right)) => Some(if need_val && *operator != AssignmentOperator::Assign {
        let left = self.transform_assignment_target_read(left, true).unwrap();
        if operator.is_logical() {
          self.ast_builder.expression_logical(*span, left, to_logical_operator(*operator), right)
        } else {
          self.ast_builder.expression_binary(*span, left, to_binary_operator(*operator), right)
        }
      } else {
        right
      }),
      (None, None) => None,
      _ => unreachable!(),
    }
  }
}

fn to_logical_operator(operator: AssignmentOperator) -> LogicalOperator {
  match operator {
    AssignmentOperator::LogicalAnd => LogicalOperator::And,
    AssignmentOperator::LogicalOr => LogicalOperator::Or,
    AssignmentOperator::LogicalNullish => LogicalOperator::Coalesce,
    _ => unreachable!(),
  }
}

fn to_binary_operator(operator: AssignmentOperator) -> BinaryOperator {
  match operator {
    AssignmentOperator::Addition => BinaryOperator::Addition,
    AssignmentOperator::Subtraction => BinaryOperator::Subtraction,
    AssignmentOperator::Multiplication => BinaryOperator::Multiplication,
    AssignmentOperator::Division => BinaryOperator::Division,
    AssignmentOperator::Remainder => BinaryOperator::Remainder,
    AssignmentOperator::Exponential => BinaryOperator::Exponential,
    AssignmentOperator::BitwiseAnd => BinaryOperator::BitwiseAnd,
    AssignmentOperator::BitwiseOR => BinaryOperator::BitwiseOR,
    AssignmentOperator::BitwiseXOR => BinaryOperator::BitwiseXOR,
    AssignmentOperator::ShiftLeft => BinaryOperator::ShiftLeft,
    AssignmentOperator::ShiftRight => BinaryOperator::ShiftRight,
    AssignmentOperator::ShiftRightZeroFill => BinaryOperator::ShiftRightZeroFill,
    _ => unreachable!(),
  }
}
