use crate::{
  analyzer::Analyzer,
  entity::{
    entity::Entity,
    unknown::{UnknownEntity, UnknownEntityKind},
  },
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, PrivateInExpression};

impl<'a> Analyzer<'a> {
  pub fn exec_private_in_expression(&mut self, node: &'a PrivateInExpression<'a>) -> Entity<'a> {
    let right = self.exec_expression(&node.right);
    UnknownEntity::new_with_deps(UnknownEntityKind::Boolean, vec![right])
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_private_in_expression(
    &self,
    node: &'a PrivateInExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let PrivateInExpression { span, left, operator, right } = node;

    let right = self.transform_expression(right, need_val);

    if need_val {
      Some(self.ast_builder.expression_private_in(*span, left.clone(), *operator, right.unwrap()))
    } else {
      right
    }
  }
}
