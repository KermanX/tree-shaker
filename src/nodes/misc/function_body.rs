use crate::{analyzer::Analyzer, ast::AstKind2, transformer::Transformer, utils::StatementVecData};
use oxc::{
  ast::ast::{ExpressionStatement, FunctionBody, Statement},
  semantic::ScopeId,
};

impl<'a> Analyzer<'a> {
  pub fn exec_function_body(&mut self, node: &'a FunctionBody<'a>) {
    let data = self.load_data::<StatementVecData>(AstKind2::FunctionBody(node));

    self.exec_statement_vec(data, &node.statements);
  }

  pub fn exec_function_expression_body(&mut self, node: &'a FunctionBody<'a>) {
    assert!(node.statements.len() == 1);
    if let Some(Statement::ExpressionStatement(expr)) = node.statements.first() {
      let dep = self.consumable(AstKind2::FunctionBody(node));
      let value = self.exec_expression(&expr.expression);
      let value = self.factory.computed(value, dep);
      let call_scope = self.call_scope_mut();
      call_scope.returned_values.push(value);
    } else {
      unreachable!();
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function_body(
    &self,
    scope_id: ScopeId,
    node: &'a FunctionBody<'a>,
  ) -> FunctionBody<'a> {
    let data = self.get_data::<StatementVecData>(AstKind2::FunctionBody(node));

    let FunctionBody { span, directives, statements } = node;

    let mut statements = self.transform_statement_vec(data, statements);

    self.patch_var_declarations(scope_id, &mut statements);

    self.ast_builder.function_body(*span, self.clone_node(directives), statements)
  }

  pub fn transform_function_expression_body(&self, node: &'a FunctionBody<'a>) -> FunctionBody<'a> {
    let need_val = self.is_referred(AstKind2::FunctionBody(node));

    let FunctionBody { span, directives, statements } = node;

    if let Some(Statement::ExpressionStatement(expr)) = statements.into_iter().next() {
      let ExpressionStatement { expression, .. } = expr.as_ref();

      let expr = self.transform_expression(expression, need_val);

      self.ast_builder.function_body(
        *span,
        self.clone_node(directives),
        self.ast_builder.vec1(self.ast_builder.statement_expression(
          *span,
          expr.unwrap_or_else(|| self.build_unused_expression(*span)),
        )),
      )
    } else {
      unreachable!();
    }
  }
}
