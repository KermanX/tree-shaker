use crate::{analyzer::Analyzer, transformer::Transformer};
use ecma_analyzer::{EcmaAnalyzer, ReturnStatementAnalyzer};
use oxc::ast::ast::{ReturnStatement, Statement};

impl<'a> ReturnStatementAnalyzer<'a> for Analyzer<'a> {
  fn on_return_value(&mut self, node: &'a ReturnStatement<'a>, value: Self::Entity) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>,
  {
    let call_scope = self.call_scope();
    let exec_dep = self.get_exec_dep(call_scope.cf_scope_depth);
    self.factory.computed(value, self.consumable((exec_dep, dep)))
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_return_statement(&self, node: &'a ReturnStatement<'a>) -> Option<Statement<'a>> {
    let need_val = self.is_referred(AstKind2::ReturnStatement(node));

    let ReturnStatement { span, argument } = node;

    Some(self.ast_builder.statement_return(
      *span,
      argument.as_ref().and_then(|arg| self.transform_expression(arg, need_val)),
    ))
  }
}
