use crate::{analyzer::Analyzer, ast::AstKind2, transformer::Transformer};
use oxc::ast::ast::{BreakStatement, Statement};

#[derive(Debug, Default)]
struct Data {
  label_used: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_break_statement(&mut self, node: &'a BreakStatement<'a>) {
    let label = node.label.as_ref().map(|label| label.name.as_str());
    if self.break_to_label(label) {
      self.set_data(AstKind2::BreakStatement(node), Data { label_used: true });
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_break_statement(&self, node: &'a BreakStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AstKind2::BreakStatement(node));

    let BreakStatement { span, label } = node;

    let label = data.label_used.then(|| self.clone_node(label)).flatten();

    Some(self.ast_builder.statement_break(*span, label))
  }
}
