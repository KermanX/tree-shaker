use super::statement_vec::StatementVecData;
use crate::{analyzer::Analyzer, ast::AstType2, transformer::Transformer};
use oxc::ast::ast::{BlockStatement, Statement};

const AST_TYPE: AstType2 = AstType2::BlockStatement;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_block_statement(&mut self, node: &'a BlockStatement) {
    let data = self.load_data::<StatementVecData>(AST_TYPE, node);
    self.exec_statement_vec(data, Some(false), &node.body);
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_block_statement(
    &mut self,
    node: BlockStatement<'a>,
  ) -> Option<Statement<'a>> {
    let data = self.get_data::<StatementVecData>(AST_TYPE, &node);

    let BlockStatement { span, body, .. } = node;

    let mut statements = self.transform_statements_vec(data, body);

    if statements.is_empty() {
      None
    } else {
      Some(self.ast_builder.statement_block(span, statements))
    }
  }
}
