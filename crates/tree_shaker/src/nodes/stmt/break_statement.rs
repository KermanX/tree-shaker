use crate::{analyzer::Analyzer, ast::AstKind2, transformer::Transformer};
use oxc::ast::ast::{BreakStatement, Statement};

impl<'a> Analyzer<'a> {
  pub fn exec_break_statement(&mut self, node: &'a BreakStatement<'a>) {
    let label = node.label.as_ref().map(|label| label.name.as_str());
    if self.break_to_label(label) {
      self.consume(AstKind2::BreakStatement(node));
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_break_statement(&self, node: &'a BreakStatement<'a>) -> Option<Statement<'a>> {
    let BreakStatement { span, label } = node;

    Some(self.ast_builder.statement_break(
      *span,
      if self.is_referred(AstKind2::BreakStatement(node)) { label.clone() } else { None },
    ))
  }
}
