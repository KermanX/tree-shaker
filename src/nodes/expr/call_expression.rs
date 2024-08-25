use crate::ast::AstType2;
use crate::entity::collector::LiteralCollector;
use crate::entity::entity::Entity;
use crate::entity::unknown::UnknownEntity;
use crate::{build_effect_from_arr, transformer::Transformer, Analyzer};
use oxc::ast::ast::{CallExpression, Expression, TSTypeParameterInstantiation};

const AST_TYPE: AstType2 = AstType2::CallExpression;

#[derive(Debug, Default)]
pub struct Data<'a> {
  need_call: bool,
  ret_collector: LiteralCollector<'a>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_call_expression(&mut self, node: &'a CallExpression) -> Entity<'a> {
    let callee = self.exec_expression(&node.callee);
    let args = self.exec_arguments(&node.arguments);

    // TODO: Track `this`. Refer https://github.com/oxc-project/oxc/issues/4341
    let (effect, ret) = callee.call(self, &UnknownEntity::new_unknown(), &args);

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.need_call |= effect;
    data.ret_collector.collect(&ret);

    ret
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
      if let Some(simple_literal) = data.ret_collector.build_expr(&self.ast_builder, span) {
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
