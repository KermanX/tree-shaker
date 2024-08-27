use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::{ContinueStatement, Statement};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_continue_statement(&mut self, node: &'a ContinueStatement<'a>) {
    self.exit_to(
      self.loop_scope_by_label(node.label.as_ref().map(|label| label.name.as_str())).cf_scope_id,
    );
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_continue_statement(
    &mut self,
    node: ContinueStatement<'a>,
  ) -> Option<Statement<'a>> {
    // TODO: strip unused label
    Some(self.ast_builder.statement_from_continue(node))
  }
}
