use crate::{
  analyzer::Analyzer,
  ast::AstKind2,
  build_effect,
  consumable::box_consumable,
  dep::ReferredDeps,
  entity::{Entity, PureCallNode},
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, NewExpression, TSTypeParameterInstantiation};

impl<'a> Analyzer<'a> {
  pub fn exec_new_expression(
    &mut self,
    node: &'a NewExpression<'a>,
    args_from_pure: Option<Entity<'a>>,
  ) -> Entity<'a> {
    if args_from_pure.is_none() && self.has_pure_notation(node) {
      let arguments = self.exec_arguments(&node.arguments);
      return self
        .factory
        .pure_result(PureCallNode::NewExpression(node, arguments), ReferredDeps::default());
    }

    let callee = self.exec_expression(&node.callee);

    let arguments = args_from_pure.unwrap_or_else(|| self.exec_arguments(&node.arguments));

    let value = callee.construct(self, box_consumable(AstKind2::NewExpression(node)), arguments);

    value
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_new_expression(
    &self,
    node: &'a NewExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let NewExpression { span, callee, arguments, .. } = node;

    if need_val || self.is_referred(AstKind2::NewExpression(node)) {
      let callee = self.transform_expression(callee, true);
      let arguments = self.transform_arguments_need_call(arguments);

      Some(self.ast_builder.expression_new(
        *span,
        callee.unwrap(),
        arguments,
        None::<TSTypeParameterInstantiation>,
      ))
    } else {
      let callee = self.transform_expression(callee, false);
      let arguments = self.transform_arguments_no_call(arguments);
      build_effect!(self.ast_builder, *span, callee, arguments)
    }
  }
}
