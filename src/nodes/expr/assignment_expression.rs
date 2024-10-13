use crate::{
  analyzer::Analyzer, ast::AstType2, build_effect, entity::Entity, scope::CfScopeKind,
  transformer::Transformer,
};
use oxc::ast::ast::{
  AssignmentExpression, AssignmentOperator, BinaryOperator, Expression, LogicalOperator,
};

const AST_TYPE: AstType2 = AstType2::AssignmentExpression;

#[derive(Debug, Default)]
pub struct DataForLogical {
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

      let conditional_dep = self.push_conditional_cf_scope(
        (AstType2::LogicalExpressionLeft, &node.left),
        CfScopeKind::LogicalExpressionRight,
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
      (None, Some(right)) => {
        if need_val && *operator != AssignmentOperator::Assign {
          if operator.is_logical() {
            let data = self.get_data::<DataForLogical>(AST_TYPE, node);

            let need_left_test_val =
              self.is_referred((AstType2::LogicalExpressionLeft, &node.left));
            let need_left_val = (need_val && data.need_left_val) || need_left_test_val;

            let left = self.transform_assignment_target_read(left, need_left_val);
            let right = data.need_right.then_some(right);

            if need_left_test_val {
              let left = left.unwrap();
              if let Some(right) = right {
                Some(self.ast_builder.expression_logical(
                  *span,
                  left,
                  to_logical_operator(*operator),
                  right,
                ))
              } else {
                Some(left)
              }
            } else {
              build_effect!(self.ast_builder, *span, left, right)
            }
          } else {
            let left = self.transform_assignment_target_read(left, true).unwrap();
            Some(self.ast_builder.expression_binary(
              *span,
              left,
              to_binary_operator(*operator),
              right,
            ))
          }
        } else {
          Some(right)
        }
      }
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
