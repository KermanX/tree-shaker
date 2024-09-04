use crate::{analyzer::Analyzer, ast::AstType2, transformer::Transformer};
use oxc::ast::ast::{ContinueStatement, Statement};

const AST_TYPE: AstType2 = AstType2::ContinueStatement;

#[derive(Debug, Default)]
struct Data {
  label_used: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_continue_statement(&mut self, node: &'a ContinueStatement<'a>) {
    let label = node.label.as_ref().map(|label| label.name.as_str());
    if self.exit_to_label(label) {
      self.set_data(AST_TYPE, node, Data { label_used: true });
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_continue_statement(&self, node: ContinueStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    Some(if data.label_used {
      self.ast_builder.statement_from_continue(node)
    } else {
      let ContinueStatement { span, .. } = node;
      self.ast_builder.statement_continue(span, None)
    })
  }
}
