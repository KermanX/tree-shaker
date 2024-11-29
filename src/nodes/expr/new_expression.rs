use crate::{
  analyzer::Analyzer,
  ast::AstKind2,
  build_effect,
  consumable::{box_consumable, ConsumableNode},
  entity::Entity,
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, NewExpression, TSTypeParameterInstantiation};

impl<'a> Analyzer<'a> {
  pub fn exec_new_expression(&mut self, node: &'a NewExpression<'a>) -> Entity<'a> {
    let pure_deps = self.has_pure_notation(node);

    let (callee, pure_deps) =
      self.exec_in_pure(pure_deps, |analyzer| analyzer.exec_expression(&node.callee));

    let arguments = self.exec_arguments(&node.arguments);

    let (instance, pure_deps) = self.exec_in_pure(pure_deps, |analyzer| {
      callee.construct(analyzer, box_consumable(AstKind2::NewExpression(node)), arguments)
    });

    self.factory.optional_computed(instance, pure_deps.map(ConsumableNode::new))
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
