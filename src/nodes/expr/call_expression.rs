use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  build_effect_from_arr,
  entity::{entity::Entity, literal::LiteralEntity, union::UnionEntity},
  transformer::Transformer,
};
use oxc::{
  allocator::{Box, IntoIn},
  ast::{
    ast::{CallExpression, Expression, TSTypeParameterInstantiation},
    AstKind,
  },
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
      let indeterminate = if node.optional {
        match callee.test_nullish() {
          Some(true) => return (Some(true), LiteralEntity::new_undefined()),
          Some(false) => false,
          None => true,
        }
      } else {
        false
      } || callee_indeterminate;

      if indeterminate {
        self.push_cf_scope_normal(None);
      }

      let args = self.exec_arguments(&node.arguments);
      let ret_val = callee.call(self, AstKind::CallExpression(node), &this, &args);

      let data = self.load_data::<Data>(AST_TYPE, node);
      data.need_optional |= indeterminate;

      if indeterminate {
        self.pop_cf_scope();
        (None, UnionEntity::new(vec![ret_val, LiteralEntity::new_undefined()]))
      } else {
        (Some(false), ret_val)
      }
    } else {
      (Some(true), LiteralEntity::new_undefined())
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

      let call_expr: Box<_> = self
        .ast_builder
        .call_expression(
          *span,
          callee,
          None::<TSTypeParameterInstantiation>,
          // Placeholder arguments
          self.ast_builder.vec(),
          data.need_optional,
        )
        .into_in(&self.allocator);

      self.deferred_arguments.borrow_mut().push((arguments, &call_expr.arguments as *const _));

      Some(Expression::CallExpression(call_expr))
    } else {
      // Only need effects in callee and args
      let callee = self.transform_callee(callee, false);
      let arguments = self.transform_arguments_no_call(arguments);
      build_effect_from_arr!(self.ast_builder, *span, vec![callee], arguments)
    }
  }
}
