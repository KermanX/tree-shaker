use crate::ast::AstType2;
use crate::entity::simple_literal::{combine_simple_literal, SimpleLiteral};
use crate::{build_effect_from_arr, entity::EntityValue, transformer::Transformer, Analyzer};
use oxc::ast::ast::{CallExpression, Expression, TSTypeParameterInstantiation};

const AST_TYPE: AstType2 = AstType2::CallExpression;

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_call: bool,
  ret_val: SimpleLiteral,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_call_expression(&mut self, node: &'a CallExpression) -> (bool, EntityValue) {
    let (callee_effect, callee_val) = self.exec_expression(&node.callee);

    let (args_effect, args_val) = self.exec_arguments(&node.arguments);

    // TODO: Track `this`. Refer https://github.com/oxc-project/oxc/issues/4341
    let (call_effect, ret_val) = callee_val.call(self, EntityValue::Unknown, args_val);

    let data = self.load_data::<Data>(AST_TYPE, node);
    combine_simple_literal(&mut data.ret_val, &ret_val);
    data.need_call |= call_effect;

    (callee_effect || args_effect || call_effect, ret_val)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_call_expression(
    &self,
    node: CallExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let CallExpression { span, callee, arguments, optional, .. } = node;

    if need_val && !data.need_call {
      if let Some(simple_literal) = self.build_simple_literal(span, &data.ret_val) {
        // Simplified to a simple literal
        let callee = self.transform_expression(callee, false);
        let arguments =
          arguments.into_iter().map(|arg| self.transform_argument_no_val(arg)).collect::<Vec<_>>();
        return build_effect_from_arr!(self.ast_builder, span, vec![callee], arguments; simple_literal);
      }
    }

    if need_val || data.need_call {
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
      // Only need effect
      let callee = self.transform_expression(callee, false);
      let arguments =
        arguments.into_iter().map(|arg| self.transform_argument_no_val(arg)).collect::<Vec<_>>();
      build_effect_from_arr!(self.ast_builder, span, vec![callee], arguments)
    }
  }
}
