use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::{ReturnStatement, Statement};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_return_statement(&mut self, node: &'a ReturnStatement) -> bool {
    let expr = node.argument.as_ref().map(|expr| self.exec_expression(expr));

    todo!()
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_return_statement(
    &self,
    node: ReturnStatement<'a>,
  ) -> Option<Statement<'a>> {
    todo!()
  }
}
