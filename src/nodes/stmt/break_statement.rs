use crate::{analyzer::Analyzer, ast::AstType2, transformer::Transformer};
use oxc::ast::ast::{BreakStatement, Statement};

const AST_TYPE: AstType2 = AstType2::BreakStatement;

#[derive(Debug, Default)]
struct Data {
  label_used: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_break_statement(&mut self, node: &'a BreakStatement<'a>) {
    let label = node.label.as_ref().map(|label| label.name.as_str());
    if self.break_to_label(label) {
      self.set_data(AST_TYPE, node, Data { label_used: true });
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_break_statement(&self, node: &'a BreakStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    Some(if data.label_used {
      self.ast_builder.statement_from_break(self.clone_node(node))
    } else {
      let BreakStatement { span, .. } = node;
      self.ast_builder.statement_break(*span, None)
    })
  }
}
