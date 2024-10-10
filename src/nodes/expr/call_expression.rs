use crate::{
  analyzer::Analyzer, ast::AstType2, build_effect_from_arr, entity::Entity,
  transformer::Transformer,
};
use oxc::ast::{
  ast::{CallExpression, Expression},
  AstKind, NONE,
};

const AST_TYPE: AstType2 = AstType2::CallExpression;

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
    if let Some((callee_indeterminate, callee, this)) = self.exec_callee(&node.callee) {
      let self_indeterminate = if node.optional {
        match callee.test_nullish() {
          Some(true) => return (Some(true), self.factory.undefined),
          Some(false) => false,
          None => true,
        }
      } else {
        false
      };

      let data = self.load_data::<Data>(AST_TYPE, node);
      data.need_optional |= self_indeterminate;

      let indeterminate = callee_indeterminate || self_indeterminate;

      if indeterminate {
        self.push_cf_scope_normal(None);
      }

      let args = self.exec_arguments(&node.arguments);
      let ret_val = callee.call(self, box_consumable(AstKind::CallExpression(node)), this, args);

      if indeterminate {
        self.pop_cf_scope();
        (None, self.factory.new_union(vec![ret_val, self.factory.undefined]))
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
    let data = self.get_data::<Data>(AST_TYPE, node);

    let CallExpression { span, callee, arguments, .. } = node;

    let need_call = need_val || self.is_referred(AstKind::CallExpression(node));

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
