use crate::{
  analyzer::Analyzer,
  ast::AstKind2,
  build_effect,
  consumable::{box_consumable, ConsumableNode},
  entity::Entity,
  transformer::Transformer,
};
use oxc::ast::{
  ast::{CallExpression, Expression},
  NONE,
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
    let pure_deps = self.has_pure_notation(node);

    let (callee_result, pure_deps) =
      self.exec_in_pure(pure_deps, |analyzer| analyzer.exec_callee(&node.callee));
    let (mut scope_count, callee, mut undefined, this) = callee_result?;

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

    let (ret_val, pure_deps) = self.exec_in_pure(pure_deps, |analyzer| {
      callee.call(analyzer, box_consumable(dep_id), this, args)
    });
    let value = self.factory.optional_computed(ret_val, pure_deps.map(ConsumableNode::new));

    Ok((scope_count, value, undefined))
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_call_expression(
    &self,
    node: &'a CallExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    self.transform_call_expression_in_chain(node, need_val, None)
  }

  pub fn transform_call_expression_in_chain(
    &self,
    node: &'a CallExpression<'a>,
    need_val: bool,
    parent_effects: Option<Expression<'a>>,
  ) -> Option<Expression<'a>> {
    let dep_id: AstKind2<'_> = AstKind2::CallExpression(node);

    let CallExpression { span, callee, arguments, optional, .. } = node;

    let need_call = need_val || self.is_referred(dep_id);

    let (need_optional, may_not_short_circuit) = self.get_chain_result(dep_id, *optional);

    if !need_call {
      let args_effect = may_not_short_circuit.then(|| self.transform_arguments_no_call(arguments));
      let all_effects = build_effect!(&self.ast_builder, *span, parent_effects, args_effect);
      return if need_optional {
        Some(self.build_chain_expression_mock(
          *span,
          self.transform_expression(callee, true).unwrap(),
          all_effects.unwrap(),
        ))
      } else {
        self.transform_expression_in_chain(callee, false, all_effects)
      };
    }

    let callee = self.transform_callee(callee, true).unwrap();

    let arguments = self.transform_arguments_need_call(arguments);

    Some(self.ast_builder.expression_call(*span, callee, NONE, arguments, need_optional))
  }
}
