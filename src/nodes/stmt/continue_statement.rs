use crate::{analyzer::Analyzer, ast::AstKind2, transformer::Transformer};
use oxc::ast::ast::{ContinueStatement, Statement};

#[derive(Debug, Default)]
struct Data {
  label_used: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_continue_statement(&mut self, node: &'a ContinueStatement<'a>) {
    let label = node.label.as_ref().map(|label| label.name.as_str());
    if self.continue_to_label(label) {
      self.set_data(AstKind2::ContinueStatement(node), Data { label_used: true });
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_continue_statement(
    &self,
    node: &'a ContinueStatement<'a>,
  ) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AstKind2::ContinueStatement(node));

    let ContinueStatement { span, .. } = node;

    Some(if data.label_used {
      Statement::ContinueStatement(self.ast_builder.alloc(self.clone_node(node)))
    } else {
      self.ast_builder.statement_continue(*span, None)
    })
  }
}
