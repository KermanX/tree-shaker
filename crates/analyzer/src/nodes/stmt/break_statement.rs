use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::{BreakStatement, Statement};

#[derive(Debug, Default)]
struct Data {
  label_used: bool,
}

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_break_statement(&mut self, node: &'a BreakStatement<'a>) {
    let label = node.label.as_ref().map(|label| label.name.as_str());
    if self.break_to_label(label) {
      self.set_data(AstKind2::BreakStatement(node), Data { label_used: true });
    }
  }
}
