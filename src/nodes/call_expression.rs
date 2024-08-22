use crate::ast_type::AstType2;
use crate::{build_effect_from_arr, entity::Entity, transformer::Transformer, Analyzer};
use oxc::{
  ast::ast::{CallExpression, Expression, TSTypeParameterInstantiation},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::CallExpression;

#[derive(Debug, Default, Clone)]
pub struct Data {
  effect: bool,
  ret_val: Entity,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_call_expression(&mut self, node: &'a CallExpression) -> (bool, Entity) {
    let callee = self.exec_expression(&node.callee);

    let args = node.arguments.iter().map(|arg| self.exec_argument(arg)).collect::<Vec<_>>();

    // TODO: Track `this`. Refer https://github.com/oxc-project/oxc/issues/4341
    // callee.call(self, Entity::Unknown, ArgumentsEntity::new(args));

    todo!()
  }
}

impl<'a> Transformer<'a> {
  fn get_effects(&self, node: CallExpression<'a>) -> Vec<Option<Expression<'a>>> {
    let callee = self.transform_expression(node.callee, false);
    let mut arguments =
      node.arguments.into_iter().map(|arg| self.transform_argument_no_val(arg)).collect::<Vec<_>>();
    arguments.insert(0, callee);
    arguments
  }

  pub(crate) fn transform_call_expression(
    &self,
    node: CallExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let span = node.span();

    if !data.effect {
      if !need_val {
        return build_effect_from_arr!(self.ast_builder, span, self.get_effects(node));
      } else if let Some(val) = self.entity_to_expression(span, &data.ret_val) {
        return build_effect_from_arr!(self.ast_builder, span, self.get_effects(node); val);
      }
    }

    let callee = self.transform_expression(node.callee, true).unwrap();
    let mut arguments = self.ast_builder.vec_with_capacity(node.arguments.len());
    for arg in node.arguments {
      arguments.push(self.transform_argument_need_val(arg));
    }
    Some(self.ast_builder.expression_call(
      span,
      arguments,
      callee,
      None::<TSTypeParameterInstantiation>,
      false,
    ))
  }
}
