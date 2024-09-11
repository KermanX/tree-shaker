use crate::ast::AstType2;
use crate::entity::entity::Entity;
use crate::entity::literal::LiteralEntity;
use crate::entity::union::UnionEntity;
use crate::entity::unknown::UnknownEntity;
use crate::{build_effect_from_arr, transformer::Transformer, Analyzer};
use oxc::ast::ast::{CallExpression, Expression, TSTypeParameterInstantiation};

const AST_TYPE: AstType2 = AstType2::CallExpression;

#[derive(Debug, Default)]
pub struct Data {
  has_effect: bool,
  need_optional: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_call_expression(&mut self, node: &'a CallExpression) -> Entity<'a> {
    let callee = self.exec_expression(&node.callee);

    let indeterminate = if node.optional {
      match callee.test_nullish() {
        Some(true) => return LiteralEntity::new_undefined(),
        Some(false) => false,
        None => true,
      }
    } else {
      false
    };

    if indeterminate {
      self.push_cf_scope_normal(None);
    }

    let args = self.exec_arguments(&node.arguments);
    // TODO: Track `this`. Refer https://github.com/oxc-project/oxc/issues/4341
    let (has_effect, ret_val) = callee.call(self, &UnknownEntity::new_unknown(), &args);

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.has_effect |= has_effect;
    data.need_optional |= indeterminate;

    if indeterminate {
      self.pop_cf_scope();
      UnionEntity::new(vec![ret_val, LiteralEntity::new_undefined()])
    } else {
      ret_val
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

    let need_call = need_val || data.has_effect;

    if need_call {
      // Need call
      let callee = self.transform_expression(callee, true).unwrap();
      let arguments = self.transform_arguments_need_call(arguments);
      Some(self.ast_builder.expression_call(
        *span,
        callee,
        None::<TSTypeParameterInstantiation>,
        arguments,
        data.need_optional,
      ))
    } else {
      // Only need effects in callee and args
      let callee = self.transform_expression(callee, false);
      let arguments = self.transform_arguments_no_call(arguments);
      build_effect_from_arr!(self.ast_builder, *span, vec![callee], arguments)
    }
  }
}
