use crate::{analyzer::Analyzer, ast::AstType2, data::StatementVecData, transformer::Transformer};
use oxc::ast::ast::BlockStatement;

const AST_TYPE: AstType2 = AstType2::BlockStatement;

impl<'a> Analyzer<'a> {
  pub fn exec_block_statement(&mut self, node: &'a BlockStatement) {
    let data = self.load_data::<StatementVecData>(AST_TYPE, node);

    self.push_cf_scope_normal(Some(false));
    self.push_variable_scope();
    self.exec_statement_vec(data, &node.body);
    self.pop_variable_scope();
    self.pop_cf_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_block_statement(
    &self,
    node: &'a BlockStatement<'a>,
  ) -> Option<BlockStatement<'a>> {
    let data = self.get_data::<StatementVecData>(AST_TYPE, node);

    let BlockStatement { span, body, .. } = node;

    let statements = self.transform_statement_vec(data, body);

    if statements.is_empty() {
      None
    } else {
      Some(self.ast_builder.block_statement(*span, statements))
    }
  }
}
