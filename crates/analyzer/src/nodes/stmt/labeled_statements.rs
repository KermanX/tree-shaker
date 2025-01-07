use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::LabeledStatement;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn declare_labeled_statement(&mut self, node: &'a LabeledStatement<'a>) {
    self.declare_statement(&node.body);
  }

  pub fn exec_labeled_statement(&mut self, node: &'a LabeledStatement<'a>) {
    self.scoping.labels.push(node);
    self.init_statement(&node.body);
  }
}
