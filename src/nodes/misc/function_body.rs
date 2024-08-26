use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::FunctionBody;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_function_body(&mut self, node: &'a FunctionBody<'a>) {
    for statement in &node.statements {
      self.exec_statement(statement);
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_function_body(&mut self, node: FunctionBody<'a>) -> FunctionBody<'a> {
    let FunctionBody { span, directives, statements, .. } = node;
    let transformed_statements = self.transform_statements(statements);
    self.ast_builder.function_body(span, directives, transformed_statements)
  }
}