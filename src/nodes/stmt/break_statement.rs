use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::BreakStatement;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_break_statement(&mut self, node: &'a BreakStatement<'a>) {
    // TODO: label
  }
}
