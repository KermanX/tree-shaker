use crate::{
  analyzer::Analyzer,
  entity::{dep::EntityDepNode, entity::Entity, function::FunctionEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{
  ArrowFunctionExpression, Expression, TSTypeAnnotation, TSTypeParameterDeclaration,
};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_arrow_function_expression(
    &mut self,
    node: &'a ArrowFunctionExpression<'a>,
  ) -> Entity<'a> {
    let dep = self.new_entity_dep(EntityDepNode::ArrowFunctionExpression(node));
    FunctionEntity::new(dep.clone())
  }

  pub(crate) fn call_arrow_function_expression(
    &mut self,
    node: &'a ArrowFunctionExpression<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> (bool, Entity<'a>) {
    self.push_variable_scope();
    self.push_function_scope();

    self.exec_formal_parameters(&node.params, args);
    if node.expression {
      self.exec_function_expression_body(&node.body);
    } else {
      self.exec_function_body(&node.body);
    }

    let has_effect = self.pop_variable_scope().has_effect;
    let ret_val = self.pop_function_scope().ret_val();
    (has_effect, ret_val)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_arrow_function_expression(
    &mut self,
    node: ArrowFunctionExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    if need_val || self.is_referred(EntityDepNode::ArrowFunctionExpression(&node)) {
      let ArrowFunctionExpression { span, expression, r#async, params, body, .. } = node;

      let params = self.transform_formal_parameters(params.unbox());
      let body = self.transform_function_body(body.unbox());

      Some(self.ast_builder.expression_arrow_function(
        span,
        expression,
        r#async,
        None::<TSTypeParameterDeclaration>,
        params,
        None::<TSTypeAnnotation>,
        body,
      ))
    } else {
      None
    }
  }
}
