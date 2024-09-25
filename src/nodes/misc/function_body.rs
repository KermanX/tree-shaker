use crate::{
  analyzer::Analyzer, ast::AstType2, data::StatementVecData, entity::forwarded::ForwardedEntity,
  transformer::Transformer,
};
use oxc::ast::{
  ast::{ExpressionStatement, FunctionBody, Statement},
  AstKind,
};

const AST_TYPE: AstType2 = AstType2::FunctionBody;

impl<'a> Analyzer<'a> {
  pub fn exec_function_body(&mut self, node: &'a FunctionBody<'a>) {
    let data = self.load_data::<StatementVecData>(AST_TYPE, node);

    self.exec_statement_vec(data, &node.statements);
  }

  pub fn exec_function_expression_body(&mut self, node: &'a FunctionBody<'a>) {
    debug_assert!(node.statements.len() == 1);
    if let Some(Statement::ExpressionStatement(expr)) = node.statements.first() {
      let dep = AstKind::FunctionBody(node);
      let value = self.exec_expression(&expr.expression);
      let call_scope = self.call_scope_mut();
      call_scope.returned_values.push(ForwardedEntity::new(value, dep));
    } else {
      unreachable!();
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function_body(&self, node: &'a FunctionBody<'a>) -> FunctionBody<'a> {
    let data = self.get_data::<StatementVecData>(AST_TYPE, node);

    let FunctionBody { span, directives, statements, .. } = node;

    let mut statements = self.transform_statement_vec(data, statements);

    self.patch_var_declarations(&mut statements);

    self.ast_builder.function_body(*span, self.clone_node(directives), statements)
  }

  pub fn transform_function_expression_body(&self, node: &'a FunctionBody<'a>) -> FunctionBody<'a> {
    let need_val = self.is_referred(AstKind::FunctionBody(&node));

    let FunctionBody { span, directives, statements, .. } = node;

    if let Some(Statement::ExpressionStatement(expr)) = statements.into_iter().next() {
      let ExpressionStatement { expression, .. } = expr.as_ref();

      let expr = self.transform_expression(expression, need_val);

      self.ast_builder.function_body(
        *span,
        self.clone_node(directives),
        expr.map_or_else(
          || self.ast_builder.vec(),
          |expr| self.ast_builder.vec1(self.ast_builder.statement_expression(*span, expr)),
        ),
      )
    } else {
      unreachable!();
    }
  }
}
