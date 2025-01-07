use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::ContinueStatement;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_continue_statement(&mut self, node: &'a ContinueStatement<'a>) {
    let label = node.label.as_ref().map(|label| label.name.as_str());
    if self.continue_to_label(label) {
      self.set_data(AstKind2::ContinueStatement(node), Data { label_used: true });
    }
  }
}
