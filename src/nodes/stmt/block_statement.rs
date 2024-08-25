use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::{BlockStatement, Statement};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_block_statement(&mut self, node: &'a BlockStatement) {
    for statement in &node.body {
      self.exec_statement(statement);
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_block_statement(
    &mut self,
    node: BlockStatement<'a>,
  ) -> Option<Statement<'a>> {
    let BlockStatement { span, body, .. } = node;
    let mut statements = self.ast_builder.vec();

    for statement in body {
      if let Some(statement) = self.transform_statement(statement) {
        statements.push(statement);
      }
    }

    if statements.is_empty() {
      None
    } else {
      Some(self.ast_builder.statement_block(span, statements))
    }
  }
}
