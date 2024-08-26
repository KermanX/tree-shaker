use crate::analyzer::Analyzer;
use oxc::ast::ast::BreakStatement;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_break_statement(&mut self, node: &'a BreakStatement<'a>) {
    // TODO: label
  }
}
