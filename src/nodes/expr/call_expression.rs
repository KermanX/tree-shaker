use crate::{
  analyzer::Analyzer, ast::AstKind2, build_effect, consumable::box_consumable, entity::Entity,
  transformer::Transformer,
};
use oxc::{
  ast::{
    ast::{CallExpression, Expression},
    NONE,
  },
  span::SPAN,
};

impl<'a> Analyzer<'a> {
  pub fn exec_call_expression(&mut self, node: &'a CallExpression) -> Entity<'a> {
    let (scope_count, value, undefined) = self.exec_call_expression_in_chain(node).unwrap();

    assert_eq!(scope_count, 0);
    assert!(undefined.is_none());

    value
  }

  /// Returns (short-circuit, value)
  pub fn exec_call_expression_in_chain(
    &mut self,
    node: &'a CallExpression,
  ) -> Result<(usize, Entity<'a>, Option<Entity<'a>>), Entity<'a>> {
    let (mut scope_count, callee, mut undefined, this) = self.exec_callee(&node.callee)?;

    let dep_id = AstKind2::CallExpression(node);

    if node.optional {
      let maybe_left = match callee.test_nullish() {
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

      self.push_logical_right_cf_scope(dep_id, callee, maybe_left, true);
      scope_count += 1;
    }

    let args = self.exec_arguments(&node.arguments);

    let ret_val = callee.call(self, box_consumable(dep_id), this, args);

    Ok((scope_count, ret_val, undefined))
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_call_expression(
    &self,
    node: &'a CallExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let dep_id: AstKind2<'_> = AstKind2::CallExpression(node);

    let CallExpression { span, callee, arguments, optional, .. } = node;

    let need_call = need_val || self.is_referred(dep_id);

    let (need_optional, may_not_short_circuit) = self.get_chain_result(dep_id, *optional);

    if !need_call {
      let args_effect = may_not_short_circuit.then(|| self.transform_arguments_no_call(arguments));
      return if need_optional {
        // FIXME: How to get the actual span?
        let args_span = SPAN;
        Some(self.build_chain_expression_mock(
          *span,
          self.transform_expression(callee, true).unwrap(),
          build_effect!(&self.ast_builder, args_span, args_effect).unwrap(),
        ))
      } else {
        build_effect!(
          &self.ast_builder,
          *span,
          self.transform_expression(callee, false),
          args_effect
        )
      };
    }

    let callee = self.transform_callee(callee, true).unwrap();

    let arguments = self.transform_arguments_need_call(arguments);

    Some(self.ast_builder.expression_call(*span, callee, NONE, arguments, need_optional))
  }
}
