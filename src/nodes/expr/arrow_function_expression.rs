use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  consumable::Consumable,
  entity::{Entity, FunctionEntitySource},
  transformer::Transformer,
};
use oxc::{
  ast::{
    ast::{ArrowFunctionExpression, Expression},
    AstKind, NONE,
  },
  semantic::ScopeId,
};
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub fn exec_arrow_function_expression(
    &mut self,
    node: &'a ArrowFunctionExpression<'a>,
  ) -> Entity<'a> {
    self.factory.new_function(
      FunctionEntitySource::ArrowFunctionExpression(node),
      self.scope_context.variable.stack.clone(),
      true,
    )
  }

  pub fn call_arrow_function_expression(
    &mut self,
    source: FunctionEntitySource<'a>,
    call_dep: Consumable<'a>,
    node: &'a ArrowFunctionExpression<'a>,
    variable_scopes: Rc<Vec<ScopeId>>,
    args: Entity<'a>,
    consume_return: bool,
  ) -> Entity<'a> {
    let parent_call_scope = self.call_scope();
    self.push_call_scope(
      source,
      call_dep,
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

    if consume_return {
      self.consume_return_values();
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

      self.call_stack.borrow_mut().push(FunctionEntitySource::ArrowFunctionExpression(node));

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
