use crate::{
  analyzer::Analyzer,
  entity::{entity::Entity, literal::LiteralEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{AssignmentTarget, Expression, StaticMemberExpression};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_static_member_expression_read(
    &mut self,
    node: &'a StaticMemberExpression<'a>,
  ) -> Entity<'a> {
    let object = self.exec_expression(&node.object);
    let key = LiteralEntity::new_string(node.property.name.as_str());
    // TODO: handle optional
    object.get_property(&key)
  }

  pub(crate) fn exec_static_member_expression_write(
    &mut self,
    node: &'a StaticMemberExpression<'a>,
    value: Entity<'a>,
  ) {
    let object = self.exec_expression(&node.object);
    let key = LiteralEntity::new_string(node.property.name.as_str());
    object.set_property(&key, value);
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_static_member_expression_read(
    &mut self,
    node: StaticMemberExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let StaticMemberExpression { span, object, property, optional, .. } = node;

    let object = self.transform_expression(object, need_val);
    object.map(|object| {
      self.ast_builder.expression_member(
        self.ast_builder.member_expression_static(span, object, property, optional),
      )
    })
  }

  pub(crate) fn transform_static_member_expression_write(
    &mut self,
    node: StaticMemberExpression<'a>,
    need_write: bool,
  ) -> Option<AssignmentTarget<'a>> {
    // TODO: side effect
    need_write.then(|| {
      self.ast_builder.assignment_target_simple(
        self.ast_builder.simple_assignment_target_member_expression(
          self.ast_builder.member_expression_from_static(node),
        ),
      )
    })
  }
}
