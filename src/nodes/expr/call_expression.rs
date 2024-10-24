use crate::{
  analyzer::Analyzer, ast::AstKind2, build_effect_from_arr, consumable::box_consumable,
  entity::Entity, transformer::Transformer,
};
use oxc::ast::{
  ast::{CallExpression, Expression},
  NONE,
};

#[derive(Debug, Default)]
pub struct Data {
  need_optional: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_call_expression(&mut self, node: &'a CallExpression) -> Entity<'a> {
    self.exec_call_expression_in_chain(node).1
  }

  /// Returns (short-circuit, value)
  pub fn exec_call_expression_in_chain(
    &mut self,
    node: &'a CallExpression,
  ) -> (Option<bool>, Entity<'a>) {
    let pure = self.has_pure_notation(node.span);

    self.scope_context.pure += pure;
    let callee = self.exec_callee(&node.callee);
    self.scope_context.pure -= pure;

    if let Some((callee_indeterminate, callee, this)) = callee {
      let self_indeterminate = if node.optional {
        match callee.test_nullish() {
          Some(true) => return (Some(true), self.factory.undefined),
          Some(false) => false,
          None => true,
        }
      } else {
        false
      };

      let data = self.load_data::<Data>(AstKind2::CallExpression(node));
      data.need_optional |= self_indeterminate;

      let indeterminate = callee_indeterminate || self_indeterminate;

      if indeterminate {
        self.push_indeterminate_cf_scope();
      }

      let args = self.exec_arguments(&node.arguments);

      self.scope_context.pure += pure;
      let ret_val = callee.call(self, box_consumable(AstKind2::CallExpression(node)), this, args);
      self.scope_context.pure -= pure;

      if indeterminate {
        self.pop_cf_scope();
        (None, self.factory.union(vec![ret_val, self.factory.undefined]))
      } else {
        (Some(false), ret_val)
      }
    } else {
      (Some(true), self.factory.undefined)
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_call_expression(
    &self,
    node: &'a CallExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AstKind2::CallExpression(node));

    let CallExpression { span, callee, arguments, .. } = node;

    let need_call = need_val || self.is_referred(AstKind2::CallExpression(node));

    if need_call {
      // Need call
      let callee = self.transform_callee(callee, true).unwrap();

      let arguments = self.transform_arguments_need_call(arguments);

      Some(self.ast_builder.expression_call(*span, callee, NONE, arguments, data.need_optional))
    } else {
      // Only need effects in callee and args
      let callee = self.transform_callee(callee, false);
      let arguments = self.transform_arguments_no_call(arguments);
      build_effect_from_arr!(self.ast_builder, *span, vec![callee], arguments)
    }
  }
}
