use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  build_effect,
  entity::{
    entity::Entity, forwarded::ForwardedEntity, literal::LiteralEntity, union::UnionEntity,
  },
  transformer::Transformer,
};
use oxc::{
  ast::{
    ast::{
      ComputedMemberExpression, Expression, MemberExpression, PrivateFieldExpression,
      StaticMemberExpression,
    },
    AstKind,
  },
  span::{GetSpan, SPAN},
};

const AST_TYPE_READ: AstType2 = AstType2::MemberExpressionRead;

#[derive(Debug, Default)]
struct Data {
  need_access: bool,
  need_optional: bool,
}

impl<'a> Analyzer<'a> {
  /// Returns (short-circuit, value, cache)
  pub fn exec_member_expression_read(
    &mut self,
    node: &'a MemberExpression<'a>,
  ) -> (Entity<'a>, Option<(Entity<'a>, Entity<'a>)>) {
    let (short_circuit, value, cache) = self.exec_member_expression_read_in_chain(node);
    debug_assert_eq!(short_circuit, Some(false));
    (value, cache)
  }

  /// Returns (short-circuit, value, cache)
  pub fn exec_member_expression_read_in_chain(
    &mut self,
    node: &'a MemberExpression<'a>,
  ) -> (Option<bool>, Entity<'a>, Option<(Entity<'a>, Entity<'a>)>) {
    let (short_circuit, object) = self.exec_expression_in_chain(node.object());

    let object_indeterminate = match short_circuit {
      Some(true) => return (Some(true), LiteralEntity::new_undefined(), None),
      Some(false) => false,
      None => true,
    };

    let indeterminate = if node.optional() {
      match object.test_nullish() {
        Some(true) => return (Some(true), LiteralEntity::new_undefined(), None),
        Some(false) => false,
        None => true,
      }
    } else {
      false
    } || object_indeterminate;
    let short_circuit = if indeterminate { None } else { Some(false) };

    let data = self.load_data::<Data>(AST_TYPE_READ, node);
    data.need_access = true;
    data.need_optional |= indeterminate;

    if indeterminate {
      self.push_cf_scope_normal(None);
    }

    let key = self.exec_key(node);
    let value = object.get_property(self, AstKind::MemberExpression(node), &key);
    let cache = Some((object, key));

    if indeterminate {
      self.pop_cf_scope();
      (short_circuit, UnionEntity::new(vec![value, LiteralEntity::new_undefined()]), cache)
    } else {
      (short_circuit, value, cache)
    }
  }

  pub fn exec_member_expression_write(
    &mut self,
    node: &'a MemberExpression<'a>,
    value: Entity<'a>,
    cache: Option<(Entity<'a>, Entity<'a>)>,
  ) {
    let dep = AstKind::MemberExpression(node);
    let value = ForwardedEntity::new(value, dep);

    let (object, key) = cache.unwrap_or_else(|| {
      let object = self.exec_expression(node.object());
      let key = self.exec_key(node);
      (object, key)
    });

    object.set_property(self, dep, &key, value);
  }

  fn exec_key(&mut self, node: &'a MemberExpression<'a>) -> Entity<'a> {
    match node {
      MemberExpression::ComputedMemberExpression(node) => self.exec_expression(&node.expression),
      MemberExpression::StaticMemberExpression(node) => {
        LiteralEntity::new_string(node.property.name.as_str())
      }
      MemberExpression::PrivateFieldExpression(node) => {
        LiteralEntity::new_string(node.field.name.as_str())
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_member_expression_read(
    &self,
    node: &'a MemberExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE_READ, node);

    if !data.need_access {
      return if need_val {
        Some(build_effect!(
          &self.ast_builder,
          node.span(),
          self.transform_expression(node.object(), false);
          self.build_undefined(SPAN)
        ))
      } else {
        build_effect!(
          &self.ast_builder,
          node.span(),
          self.transform_expression(node.object(), false)
        )
      };
    }

    let need_read = need_val || self.is_referred(AstKind::MemberExpression(node));

    match node {
      MemberExpression::ComputedMemberExpression(node) => {
        let ComputedMemberExpression { span, object, expression, .. } = node.as_ref();

        let object = self.transform_expression(object, need_read);
        let key = self.transform_expression(expression, need_read);
        if need_read {
          Some(self.ast_builder.expression_member(self.ast_builder.member_expression_computed(
            *span,
            object.unwrap(),
            key.unwrap(),
            data.need_optional,
          )))
        } else {
          build_effect!(&self.ast_builder, *span, object, key)
        }
      }
      MemberExpression::StaticMemberExpression(node) => {
        let StaticMemberExpression { span, object, property, .. } = node.as_ref();

        let object = self.transform_expression(object, need_read);
        if need_read {
          Some(self.ast_builder.expression_member(self.ast_builder.member_expression_static(
            *span,
            object.unwrap(),
            property.clone(),
            data.need_optional,
          )))
        } else {
          object
        }
      }
      MemberExpression::PrivateFieldExpression(node) => {
        let PrivateFieldExpression { span, object, field, .. } = node.as_ref();

        let object = self.transform_expression(object, need_read);

        if need_read {
          Some(self.ast_builder.expression_member(
            self.ast_builder.member_expression_private_field_expression(
              *span,
              object.unwrap(),
              field.clone(),
              data.need_optional,
            ),
          ))
        } else {
          object
        }
      }
    }
  }

  pub fn transform_member_expression_write(
    &self,
    node: &'a MemberExpression<'a>,
  ) -> Option<MemberExpression<'a>> {
    let need_write = self.is_referred(AstKind::MemberExpression(node));

    match node {
      MemberExpression::ComputedMemberExpression(node) => {
        let ComputedMemberExpression { span, object, expression, .. } = node.as_ref();

        let transformed_object = self.transform_expression(object, need_write);
        let transformed_key = self.transform_expression(expression, need_write);
        if need_write {
          Some(self.ast_builder.member_expression_computed(
            *span,
            transformed_object.unwrap(),
            transformed_key.unwrap(),
            false,
          ))
        } else if transformed_object.is_some() || transformed_key.is_some() {
          Some(self.ast_builder.member_expression_computed(
            *span,
            self.transform_expression(object, true).unwrap(),
            self.transform_expression(expression, true).unwrap(),
            false,
          ))
        } else {
          None
        }
      }
      MemberExpression::StaticMemberExpression(node) => {
        let StaticMemberExpression { span, object, property, .. } = node.as_ref();

        let transformed_object = self.transform_expression(object, need_write);
        if need_write {
          Some(self.ast_builder.member_expression_static(
            *span,
            transformed_object.unwrap(),
            property.clone(),
            false,
          ))
        } else if transformed_object.is_some() {
          Some(self.ast_builder.member_expression_static(
            *span,
            self.transform_expression(object, true).unwrap(),
            property.clone(),
            false,
          ))
        } else {
          None
        }
      }
      MemberExpression::PrivateFieldExpression(node) => {
        let PrivateFieldExpression { span, object, field, .. } = node.as_ref();

        let transformed_object = self.transform_expression(object, need_write);

        if need_write {
          Some(self.ast_builder.member_expression_private_field_expression(
            *span,
            transformed_object.unwrap(),
            field.clone(),
            false,
          ))
        } else if transformed_object.is_some() {
          Some(self.ast_builder.member_expression_private_field_expression(
            *span,
            self.transform_expression(object, true).unwrap(),
            field.clone(),
            false,
          ))
        } else {
          None
        }
      }
    }
  }
}
