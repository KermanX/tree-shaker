use crate::{analyzer::Analyzer, ast::AstKind2, data::StatementVecData, transformer::Transformer};
use oxc::ast::ast::StaticBlock;

impl<'a> Analyzer<'a> {
  pub fn exec_static_block(&mut self, node: &'a StaticBlock<'a>) {
    let data = self.load_data::<StatementVecData>(AstKind2::StaticBlock(node));

    self.exec_statement_vec(data, &node.body);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_static_block(&self, node: &'a StaticBlock<'a>) -> Option<StaticBlock<'a>> {
    let data = self.get_data::<StatementVecData>(AstKind2::StaticBlock(node));

    let StaticBlock { span, body, .. } = node;

    let body = self.transform_statement_vec(data, body);

    (!body.is_empty()).then(|| self.ast_builder.static_block(*span, body))
  }
}
