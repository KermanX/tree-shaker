use crate::{
  analyzer::Analyzer,
  ast::AstKind2,
  build_effect,
  consumable::box_consumable,
  entity::{Entity, PureCallNode},
  transformer::Transformer,
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
    self.exec_call_expression_in_chain(node, None).1
  }

  /// Returns (short-circuit, value)
  pub fn exec_call_expression_in_chain(
    &mut self,
    node: &'a CallExpression,
    args_from_pure: Option<Entity<'a>>,
  ) -> (Option<bool>, Entity<'a>) {
    if args_from_pure.is_none() && self.has_pure_notation(node.span) {
      let args = self.exec_arguments(&node.arguments);
      return self.factory.pure_result(PureCallNode::CallExpression(node, args));
    }

    let callee = self.exec_callee(&node.callee);

    if let Some((callee_indeterminate, callee, this)) = callee {
      let self_indeterminate = if node.optional {
        match callee.test_nullish(self) {
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

      let args = args_from_pure.unwrap_or_else(|| self.exec_arguments(&node.arguments));

      let ret_val = callee.call(self, box_consumable(AstKind2::CallExpression(node)), this, args);

      if indeterminate {
        self.pop_cf_scope();
        (None, self.factory.union((ret_val, self.factory.undefined)))
      } else {
        (Some(false), ret_val)
      }
    } else {
      (Some(true), self.factory.undefined)
    }
  }

  pub fn exec_call_expression_by_pure(
    &mut self,
    node: &'a CallExpression,
    arguments: Entity<'a>,
  ) -> Entity<'a> {
    let callee = self.exec_callee(&node.callee);

    if let Some((callee_indeterminate, callee, this)) = callee {
      let self_indeterminate = if node.optional {
        match callee.test_nullish(self) {
          Some(true) => return self.factory.undefined,
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

      let ret_val =
        callee.call(self, box_consumable(AstKind2::CallExpression(node)), this, arguments);

      if indeterminate {
        self.pop_cf_scope();
        self.factory.union((ret_val, self.factory.undefined))
      } else {
        ret_val
      }
    } else {
      self.factory.undefined
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
      build_effect!(self.ast_builder, *span, callee, arguments)
    }
  }
}
