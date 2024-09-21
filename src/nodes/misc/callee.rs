use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{
    dep::EntityDepNode, entity::Entity, forwarded::ForwardedEntity, literal::LiteralEntity,
  },
  transformer::Transformer,
};
use oxc::{
  ast::{
    ast::{Expression, MemberExpression},
    match_member_expression,
  },
  span::{GetSpan, SPAN},
};

const AST_TYPE: AstType2 = AstType2::Callee;

fn unwrap_to_member_expression<'a>(node: &'a Expression<'a>) -> Option<&'a MemberExpression<'a>> {
  match node {
    match_member_expression!(Expression) => Some(node.to_member_expression()),
    Expression::ParenthesizedExpression(node) => unwrap_to_member_expression(&node.expression),
    _ => None,
  }
}

impl<'a> Analyzer<'a> {
  /// Returns: Some((callee, this)) or None for should not call due to ?. operator
  pub fn exec_callee(&mut self, node: &'a Expression<'a>) -> Option<(Entity<'a>, Entity<'a>)> {
    let dep: EntityDepNode = (AST_TYPE, node).into();
    if let Some(member_expr) = unwrap_to_member_expression(node) {
      let (callee, cache) = self.exec_member_expression_read(member_expr);
      cache.map(|(object, _)| (callee, ForwardedEntity::new(object, dep)))
    } else {
      let callee = self.exec_expression(node);
      Some((callee, ForwardedEntity::new(LiteralEntity::new_undefined(), dep)))
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

      let is_referred = self.is_referred((AST_TYPE, node));
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
