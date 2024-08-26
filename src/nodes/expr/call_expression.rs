use crate::ast::AstType2;
use crate::entity::entity::Entity;
use crate::entity::unknown::UnknownEntity;
use crate::{build_effect_from_arr, transformer::Transformer, Analyzer};
use oxc::ast::ast::{CallExpression, Expression, TSTypeParameterInstantiation};

const AST_TYPE: AstType2 = AstType2::CallExpression;

#[derive(Debug, Default)]
pub struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_call_expression(&mut self, node: &'a CallExpression) -> Entity<'a> {
    let callee = self.exec_expression(&node.callee);
    let args = self.exec_arguments(&node.arguments);

    // TODO: Track `this`. Refer https://github.com/oxc-project/oxc/issues/4341
    let (has_effect, ret_val) = callee.call(self, &UnknownEntity::new_unknown(), &args);

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.has_effect |= has_effect;

    ret_val
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_call_expression(
    &mut self,
    node: CallExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let CallExpression { span, callee, arguments, optional, .. } = node;

    let need_call = data.has_effect;

    if need_val || need_call {
      // Need call
      let callee = self.transform_expression(callee, true).unwrap();
      let mut transformed_arguments = self.ast_builder.vec();
      for arg in arguments {
        transformed_arguments.push(self.transform_argument_need_val(arg));
      }
      Some(self.ast_builder.expression_call(
        span,
        transformed_arguments,
        callee,
        None::<TSTypeParameterInstantiation>,
        optional,
      ))
    } else {
      // Only need effects in callee and args
      let callee = self.transform_expression(callee, false);
      let arguments =
        arguments.into_iter().map(|arg| self.transform_argument_no_val(arg)).collect::<Vec<_>>();
      build_effect_from_arr!(self.ast_builder, span, vec![callee], arguments)
    }
  }
}
