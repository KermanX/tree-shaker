use crate::{
  build_effect_from_arr,
  entity::{arguments::ArgumentsEntity, Entity},
  TreeShaker,
};
use oxc::{
  ast::ast::{CallExpression, Expression, TSTypeParameterInstantiation},
  span::GetSpan,
};

#[derive(Debug, Default, Clone)]
pub struct Data {
  pure: bool,
  val: Entity,
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_call_expression(&mut self, node: &'a CallExpression) -> Entity {
    let data = self.load_data::<Data>(node);

    let callee = self.exec_expression(&node.callee);

    let args = node.arguments.iter().map(|arg| self.exec_argument(arg)).collect::<Vec<_>>();

    // TODO: Track `this`. Refer https://github.com/oxc-project/oxc/issues/4341
    let (pure, val) = callee.call(Entity::Unknown, ArgumentsEntity::new(args));

    data.pure = pure;
    data.val = val.clone();

    val
  }

  fn get_effects(&mut self, node: CallExpression<'a>) -> Vec<Option<Expression<'a>>> {
    let callee = self.transform_expression(node.callee, false);
    let mut arguments =
      node.arguments.into_iter().map(|arg| self.transform_argument_no_val(arg)).collect::<Vec<_>>();
    arguments.insert(0, callee);
    arguments
  }

  pub(crate) fn transform_call_expression(
    &mut self,
    node: CallExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.load_data::<Data>(&node);

    let span = node.span();

    if data.pure {
      if !need_val {
        return build_effect_from_arr!(self.ast_builder, span, self.get_effects(node));
      } else if let Some(val) = self.entity_to_expression(span, &data.val) {
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
