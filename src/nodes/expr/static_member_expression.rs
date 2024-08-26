use crate::{
  analyzer::Analyzer,
  entity::{entity::Entity, literal::LiteralEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, StaticMemberExpression};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_static_member_expression(
    &mut self,
    node: &'a StaticMemberExpression<'a>,
  ) -> Entity<'a> {
    let object = self.exec_expression(&node.object);
    let property = LiteralEntity::new_string(node.property.name.as_str());
    // TODO: handle optional
    object.get_property(&property)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_static_member_expression(
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
}
