use crate::transformer::Transformer;
use oxc::{
  ast::ast::{Expression, Program},
  semantic::SemanticBuilder,
  span::SPAN,
};
use oxc_traverse::{traverse_mut, Traverse, TraverseCtx};
use std::mem::replace;

#[derive(Default)]
struct Patch {}

impl<'a> Traverse<'a> for Patch {
  fn exit_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
    // Replace `(a = 1, 1)` with `a = 1`
    let Expression::SequenceExpression(seq) = node else {
      return;
    };
    if seq.expressions.len() != 2 {
      return;
    }
    let first = &seq.expressions[0];
    let second = &seq.expressions[1];
    let Expression::AssignmentExpression(assignment) = first else {
      return;
    };
    let assignment_rhs = &assignment.right;
    if !is_same_literal(assignment_rhs, second) {
      return;
    }
    let assignment = replace(&mut seq.expressions[0], ctx.ast.expression_null_literal(SPAN));
    *node = assignment;
  }
}

fn is_same_literal(a: &Expression, b: &Expression) -> bool {
  match (a, b) {
    (Expression::NumericLiteral(a), Expression::NumericLiteral(b)) => a.value == b.value,
    (Expression::StringLiteral(a), Expression::StringLiteral(b)) => a.value == b.value,
    (Expression::BooleanLiteral(a), Expression::BooleanLiteral(b)) => a.value == b.value,
    (Expression::NullLiteral(_), Expression::NullLiteral(_)) => true,
    (Expression::UnaryExpression(a), Expression::UnaryExpression(b)) => {
      a.operator == b.operator && is_same_literal(&a.argument, &b.argument)
    }
    _ => false,
  }
}

impl<'a> Transformer<'a> {
  pub fn patch_ast(&self, program: &mut Program<'a>) {
    let mut patch = Patch::default();
    let (symbols, scopes) =
      SemanticBuilder::new().build(program).semantic.into_symbol_table_and_scope_tree();
    traverse_mut(&mut patch, self.allocator, program, symbols, scopes);
  }
}
