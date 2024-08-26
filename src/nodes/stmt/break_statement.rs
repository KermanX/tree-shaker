use crate::analyzer::Analyzer;
use oxc::ast::ast::BreakStatement;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_break_statement(&mut self, node: &'a BreakStatement<'a>) {
    self.exit_to(
      self.loop_scope_by_label(node.label.as_ref().map(|label| label.name.as_str())).cf_scope_id,
    );
  }
}
