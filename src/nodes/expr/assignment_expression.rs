use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  build_effect,
  entity::{Entity, EntityDepNode},
  scope::{conditional::ConditionalData, CfScopeKind},
  transformer::Transformer,
};
use oxc::ast::{
  ast::{AssignmentExpression, AssignmentOperator, BinaryOperator, Expression, LogicalOperator},
  AstKind,
};

const AST_TYPE: AstType2 = AstType2::AssignmentExpression;

#[derive(Debug, Default)]
pub struct DataForLogical<'a> {
  need_left_val: bool,
  need_right: bool,
  conditional: ConditionalData<'a>,
}

impl<'a> Analyzer<'a> {
  pub fn exec_assignment_expression(&mut self, node: &'a AssignmentExpression<'a>) -> Entity<'a> {
    if node.operator == AssignmentOperator::Assign {
      let rhs = self.exec_expression(&node.right);
      self.exec_assignment_target_write(&node.left, rhs.clone(), None);
      rhs
    } else if node.operator.is_logical() {
      let (left, cache) = self.exec_assignment_target_read(&node.left);

      let (need_left_val, need_right) = match &node.operator {
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

      let data = self.load_data::<DataForLogical>(AST_TYPE, node);

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
      self.push_cf_scope_for_dep(AstKind::AssignmentExpression(node));

      let value = match (need_left_val, need_right) {
        (false, true) => self.exec_expression(&node.right),
        (true, false) => left,
        (true, true) => {
          let right = self.exec_expression(&node.right);
          self.factory.new_union(vec![left, right])
        }
        (false, false) => unreachable!(),
      };

      self.pop_cf_scope();
      self.pop_cf_scope();

      if need_right {
        self.exec_assignment_target_write(&node.left, value.clone(), cache);
      }

      value
    } else {
      let (lhs, cache) = self.exec_assignment_target_read(&node.left);
      let rhs = self.exec_expression(&node.right);
      let value = self.entity_op.binary_op(self, to_binary_operator(node.operator), lhs, rhs);
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
      (Some(left), Some(right)) => Some(self.ast_builder.expression_assignment(
        *span,
        if operator.is_logical() {
          let data = self.get_data::<DataForLogical>(AST_TYPE, node);

          if data.need_left_val {
            *operator
          } else {
            AssignmentOperator::Assign
          }
        } else {
          *operator
        },
        left,
        right,
      )),
      (None, Some(right)) => Some(if need_val && *operator != AssignmentOperator::Assign {
        if operator.is_logical() {
          let data = self.get_data::<DataForLogical>(AST_TYPE, node);

          let need_left_val = (need_val && data.need_left_val)
            || self.is_referred(AstKind::AssignmentExpression(node));

          let left = self.transform_assignment_target_read(left, need_left_val);
          let right = data.need_right.then_some(right);

          match (left, right) {
            (Some(left), Some(right)) => {
              if need_left_val {
                self.ast_builder.expression_logical(
                  *span,
                  left,
                  to_logical_operator(*operator),
                  right,
                )
              } else {
                build_effect!(self.ast_builder, *span, Some(left); right)
              }
            }
            (Some(left), None) => left,
            (None, Some(right)) => right,
            (None, None) => unreachable!(),
          }
        } else {
          let left = self.transform_assignment_target_read(left, true).unwrap();
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
