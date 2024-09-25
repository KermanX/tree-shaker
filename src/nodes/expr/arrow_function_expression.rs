use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  entity::{
    dep::{EntityDep, EntityDepNode},
    entity::Entity,
    function::{FunctionEntity, FunctionEntitySource},
  },
  scope::variable_scope::VariableScopes,
  transformer::Transformer,
};
use oxc::ast::{
  ast::{ArrowFunctionExpression, Expression},
  AstKind, NONE,
};
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub fn exec_arrow_function_expression(
    &mut self,
    node: &'a ArrowFunctionExpression<'a>,
  ) -> Entity<'a> {
    FunctionEntity::new(
      FunctionEntitySource::ArrowFunctionExpression(node),
      self.scope_context.variable_scopes.clone(),
      true,
    )
  }

  pub fn call_arrow_function_expression(
    &mut self,
    source: EntityDepNode,
    dep: EntityDep,
    node: &'a ArrowFunctionExpression<'a>,
    variable_scopes: Rc<VariableScopes<'a>>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let parent_call_scope = self.call_scope();
    self.push_call_scope(
      source,
      dep,
      variable_scopes,
      parent_call_scope.this.clone(),
      parent_call_scope.args.clone(),
      node.r#async,
      false,
    );

    self.exec_formal_parameters(&node.params, args, DeclarationKind::ArrowFunctionParameter);
    if node.expression {
      self.exec_function_expression_body(&node.body);
    } else {
      self.exec_function_body(&node.body);
    }

    self.pop_call_scope()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_arrow_function_expression(
    &self,
    node: &'a ArrowFunctionExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    if need_val || self.is_referred(AstKind::ArrowFunctionExpression(node)) {
      let ArrowFunctionExpression { span, expression, r#async, params, body, .. } = node;

      self.call_stack.borrow_mut().push(AstKind::ArrowFunctionExpression(node).into());

      let params = self.transform_formal_parameters(params);
      let body = if *expression {
        self.transform_function_expression_body(body)
      } else {
        self.transform_function_body(body)
      };

      self.call_stack.borrow_mut().pop();

      Some(self.ast_builder.expression_arrow_function(
        *span,
        *expression,
        *r#async,
        NONE,
        params,
        NONE,
        body,
      ))
    } else {
      None
    }
  }
}
