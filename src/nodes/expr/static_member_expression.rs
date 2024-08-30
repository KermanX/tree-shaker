use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{dep::EntityDep, entity::Entity, forwarded::ForwardedEntity, literal::LiteralEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{AssignmentTarget, Expression, StaticMemberExpression};

const AST_TYPE: AstType2 = AstType2::StaticMemberExpression;

#[derive(Debug, Default)]
struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_static_member_expression_read(
    &mut self,
    node: &'a StaticMemberExpression<'a>,
  ) -> Entity<'a> {
    let object = self.exec_expression(&node.object);
    let key = LiteralEntity::new_string(node.property.name.as_str());
    // TODO: handle optional
    let (has_effect, value) = object.get_property(self, &key);

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.has_effect |= has_effect;

    value
  }

  pub(crate) fn exec_static_member_expression_write(
    &mut self,
    node: &'a StaticMemberExpression<'a>,
    value: Entity<'a>,
    dep: EntityDep<'a>,
  ) {
    let object = self.exec_expression(&node.object);
    let key = LiteralEntity::new_string(node.property.name.as_str());
    let has_effect = object.set_property(self, &key, ForwardedEntity::new(value, dep));

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.has_effect |= has_effect;
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_static_member_expression_read(
    &mut self,
    node: StaticMemberExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let StaticMemberExpression { span, object, property, optional, .. } = node;

    let need_read = need_val || data.has_effect;

    let object = self.transform_expression(object, need_read);
    if need_read {
      object.map(|object| {
        self.ast_builder.expression_member(
          self.ast_builder.member_expression_static(span, object, property, optional),
        )
      })
    } else {
      object
    }
  }

  pub(crate) fn transform_static_member_expression_write(
    &mut self,
    node: StaticMemberExpression<'a>,
    need_write: bool,
  ) -> Option<AssignmentTarget<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let need_write = need_write || data.has_effect;

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
