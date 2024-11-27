use crate::{
  analyzer::Analyzer, ast::AstKind2, build_effect, consumable::box_consumable, entity::Entity,
  transformer::Transformer,
};
use oxc::{
  ast::ast::{
    ComputedMemberExpression, Expression, MemberExpression, PrivateFieldExpression,
    StaticMemberExpression,
  },
  span::{GetSpan, SPAN},
};

#[derive(Debug, Default)]
struct DataRead {
  need_access: bool,
  need_optional: bool,
}

impl<'a> Analyzer<'a> {
  /// Returns (short-circuit, value, cache)
  pub fn exec_member_expression_read(
    &mut self,
    node: &'a MemberExpression<'a>,
    will_write: bool,
  ) -> (Entity<'a>, Option<(Entity<'a>, Entity<'a>)>) {
    let (short_circuit, value, cache) = self.exec_member_expression_read_in_chain(node, will_write);
    debug_assert_eq!(short_circuit, Some(false));
    (value, cache)
  }

  /// Returns (short-circuit, value, cache)
  pub fn exec_member_expression_read_in_chain(
    &mut self,
    node: &'a MemberExpression<'a>,
    will_write: bool,
  ) -> (Option<bool>, Entity<'a>, Option<(Entity<'a>, Entity<'a>)>) {
    let (short_circuit, object) = self.exec_expression_in_chain(node.object());

    let object_indeterminate = match short_circuit {
      Some(true) => return (Some(true), self.factory.undefined, None),
      Some(false) => false,
      None => true,
    };

    let self_indeterminate = if node.optional() {
      match object.test_nullish(self) {
        Some(true) => return (Some(true), self.factory.undefined, None),
        Some(false) => false,
        None => true,
      }
    } else {
      false
    };

    let data = self.load_data::<DataRead>(AstKind2::MemberExpressionRead(node));
    data.need_access = true;
    data.need_optional |= self_indeterminate;

    let indeterminate = object_indeterminate || self_indeterminate;

    if indeterminate {
      self.push_indeterminate_cf_scope();
    }

    if will_write {
      self.push_dependent_cf_scope(object.clone());
    }
    let key = self.exec_key(node);
    if will_write {
      self.pop_cf_scope();
    }

    let value = object.get_property(self, box_consumable(AstKind2::MemberExpression(node)), key);
    let cache = Some((object, key));

    if indeterminate {
      self.pop_cf_scope();
      (None, self.factory.union((value, self.factory.undefined)), cache)
    } else {
      (Some(false), value, cache)
    }
  }

  pub fn exec_member_expression_write(
    &mut self,
    node: &'a MemberExpression<'a>,
    value: Entity<'a>,
    cache: Option<(Entity<'a>, Entity<'a>)>,
  ) {
    let (object, key) = cache.unwrap_or_else(|| {
      let object = self.exec_expression(node.object());

      self.push_dependent_cf_scope(object.clone());
      let key = self.exec_key(node);
      self.pop_cf_scope();

      (object, key)
    });

    object.set_property(self, box_consumable(AstKind2::MemberExpression(node)), key, value);
  }

  fn exec_key(&mut self, node: &'a MemberExpression<'a>) -> Entity<'a> {
    match node {
      MemberExpression::ComputedMemberExpression(node) => self.exec_expression(&node.expression),
      MemberExpression::StaticMemberExpression(node) => {
        self.factory.string(node.property.name.as_str())
      }
      MemberExpression::PrivateFieldExpression(node) => {
        self.factory.string(self.escape_private_identifier_name(node.field.name.as_str()))
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
    let data = self.get_data::<DataRead>(AstKind2::MemberExpressionRead(node));

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

    let need_read = need_val || self.is_referred(AstKind2::MemberExpression(node));

    match node {
      MemberExpression::ComputedMemberExpression(node) => {
        let ComputedMemberExpression { span, object, expression, .. } = node.as_ref();

        if need_read {
          let object = self.transform_expression(object, true).unwrap();
          let key = self.transform_expression(expression, true).unwrap();
          Some(Expression::from(self.ast_builder.member_expression_computed(
            *span,
            object,
            key,
            data.need_optional,
          )))
        } else {
          let object = self.transform_expression(object, false);
          let key = self.transform_expression(expression, false);
          build_effect!(&self.ast_builder, *span, object, key)
        }
      }
      MemberExpression::StaticMemberExpression(node) => {
        let StaticMemberExpression { span, object, property, .. } = node.as_ref();

        let object = self.transform_expression(object, need_read);
        if need_read {
          Some(Expression::from(self.ast_builder.member_expression_static(
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
          Some(
            self
              .ast_builder
              .member_expression_private_field_expression(
                *span,
                object.unwrap(),
                field.clone(),
                data.need_optional,
              )
              .into(),
          )
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
    let need_write = self.is_referred(AstKind2::MemberExpression(node));

    match node {
      MemberExpression::ComputedMemberExpression(node) => {
        let ComputedMemberExpression { span, object, expression, .. } = node.as_ref();

        let transformed_object = self.transform_expression(object, need_write);

        let need_key_value = need_write || transformed_object.is_some();
        let transformed_key = self.transform_expression(expression, need_key_value);

        if need_key_value {
          Some(self.ast_builder.member_expression_computed(
            *span,
            transformed_object.unwrap(),
            transformed_key.unwrap(),
            false,
          ))
        } else if transformed_key.is_some() {
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
