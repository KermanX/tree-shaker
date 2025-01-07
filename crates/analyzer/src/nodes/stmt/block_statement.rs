use crate::{analyzer::Analyzer, host::Host, scoping::CfScopeKind};
use oxc::ast::{ast::BlockStatement, AstKind};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_block_statement(&mut self, node: &'a BlockStatement) {
    let labels = self.take_labels();

    self.push_cf_scope(CfScopeKind::Block, labels, Some(false));
    self.init_statement_vec(AstKind::BlockStatement(node), &node.body);
    self.pop_cf_scope();
  }
}
