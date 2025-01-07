use crate::{host::Host, 
  analyzer::Analyzer,  
};
use oxc::{
  ast::ast::{
    ComputedMemberExpression, Expression, MemberExpression, PrivateFieldExpression,
    StaticMemberExpression,
  },
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  /// Returns (short-circuit, value, cache)
  pub fn exec_member_expression_read(
    &mut self,
    node: &'a MemberExpression<'a>,
    will_write: bool,
  ) -> (H::Entity, (H::Entity, H::Entity)) {
    let (scope_count, value, undefined, cache) =
      self.exec_member_expression_read_in_chain(node, will_write).unwrap();

    assert_eq!(scope_count, 0);
    assert!(undefined.is_none());

    (value, cache)
  }

  /// Returns (scope_count, value, forwarded_undefined, cache)
  pub fn exec_member_expression_read_in_chain(
    &mut self,
    node: &'a MemberExpression<'a>,
    will_write: bool,
  ) -> Result<(usize, H::Entity, Option<H::Entity>, (H::Entity, H::Entity)), H::Entity> {
    let (mut scope_count, object, mut undefined) = self.exec_expression_in_chain(node.object())?;

    let dep_id = AstKind2::MemberExpression(node);

    if node.optional() {
      let maybe_left = match object.test_nullish() {
        Some(true) => {
          self.pop_multiple_cf_scopes(scope_count);
          return Err(self.forward_logical_left_val(dep_id, self.factory.undefined, true, false));
        }
        Some(false) => false,
        None => {
          undefined = Some(self.forward_logical_left_val(
            dep_id,
            undefined.unwrap_or(self.factory.undefined),
            true,
            false,
          ));
          true
        }
      };

      self.push_logical_right_cf_scope(dep_id, object, maybe_left, true);
      scope_count += 1;
    }

    if will_write {
      self.push_dependent_cf_scope(object);
    }
    let key = self.exec_key(node);
    if will_write {
      self.pop_cf_scope();
    }

    let value = object.get_property(self, self.consumable(dep_id), key);

    Ok((scope_count, value, undefined, (object, key)))
  }

  pub fn exec_member_expression_write(
    &mut self,
    node: &'a MemberExpression<'a>,
    value: H::Entity,
    cache: Option<(H::Entity, H::Entity)>,
  ) {
    let (object, key) = cache.unwrap_or_else(|| {
      let object = self.exec_expression(node.object());

      self.push_dependent_cf_scope(object);
      let key = self.exec_key(node);
      self.pop_cf_scope();

      (object, key)
    });

    object.set_property(self, self.consumable(AstKind2::MemberExpression(node)), key, value);
  }

  fn exec_key(&mut self, node: &'a MemberExpression<'a>) -> H::Entity {
    match node {
      MemberExpression::ComputedMemberExpression(node) => self.exec_expression(&node.expression),
      MemberExpression::StaticMemberExpression(node) => self.exec_identifier_name(&node.property),
      MemberExpression::PrivateFieldExpression(node) => self.exec_private_identifier(&node.field),
    }
  }
}

