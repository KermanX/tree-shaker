use crate::{
  analyzer::Analyzer, ast::AstKind2, consumable::box_consumable, entity::Entity,
  transformer::Transformer,
};
use oxc::{
  ast::{
    ast::{ChainElement, Expression, MemberExpression},
    match_member_expression,
  },
  span::{GetSpan, SPAN},
};

fn unwrap_to_member_expression<'a>(node: &'a Expression<'a>) -> Option<&'a MemberExpression<'a>> {
  match node {
    match_member_expression!(Expression) => Some(node.to_member_expression()),
    Expression::ParenthesizedExpression(node) => unwrap_to_member_expression(&node.expression),
    Expression::ChainExpression(node) => match &node.expression {
      match_member_expression!(ChainElement) => Some(node.expression.to_member_expression()),
      _ => None,
    },
    _ => None,
  }
}

impl<'a> Analyzer<'a> {
  /// Returns: Ok((scope_count, callee, undefined, this)) or Err(forwarded_undefined) for should not call due to ?. operator
  pub fn exec_callee(
    &mut self,
    node: &'a Expression<'a>,
  ) -> Result<(usize, Entity<'a>, Option<Entity<'a>>, Entity<'a>), Entity<'a>> {
    let dep = box_consumable(AstKind2::Callee(node));
    if let Some(member_expr) = unwrap_to_member_expression(node) {
      let (scope_count, callee, undefined, (object, _)) =
        self.exec_member_expression_read_in_chain(member_expr, false)?;
      Ok((scope_count, callee, undefined, self.factory.computed(object, dep)))
    } else {
      let (scope_count, callee, undefined) = self.exec_expression_in_chain(node)?;
      Ok((scope_count, callee, undefined, self.factory.computed(self.factory.undefined, dep)))
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_callee(
    &self,
    node: &'a Expression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    if need_val {
      let transformed_expr = self.transform_expression(node, true).unwrap();

      let is_referred = self.is_referred(AstKind2::Callee(node));
      let was_member_expression = unwrap_to_member_expression(node).is_some();
      let is_member_expression = unwrap_to_member_expression(&transformed_expr).is_some();
      Some(if is_referred && !was_member_expression && is_member_expression {
        self.ast_builder.expression_sequence(transformed_expr.span(), {
          let mut seq = self.ast_builder.vec_with_capacity(2);
          seq.push(self.build_unused_expression(SPAN));
          seq.push(transformed_expr);
          seq
        })
      } else {
        transformed_expr
      })
    } else {
      self.transform_expression(node, false)
    }
  }
}
