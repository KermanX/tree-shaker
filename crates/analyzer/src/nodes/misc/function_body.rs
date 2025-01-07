use crate::{host::Host, analyzer::Analyzer,  utils::StatementVecData};
use oxc::{
  ast::ast::{ExpressionStatement, FunctionBody, Statement},
  semantic::ScopeId,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_function_body(&mut self, node: &'a FunctionBody<'a>) {
   
    self.init_statement_vec(data, &node.statements);
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

