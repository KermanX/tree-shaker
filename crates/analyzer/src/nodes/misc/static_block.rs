use crate::{analyzer::Analyzer,  host::Host, utils::StatementVecData};
use oxc::{allocator, ast::ast::StaticBlock};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_static_block(&mut self, node: &'a StaticBlock<'a>) {
    self.init_statement_vec(data, &node.body);
  }
}
