use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::{Statement, TryStatement};

impl<'a> Analyzer<'a> {
  pub fn exec_try_statement(&mut self, node: &'a TryStatement<'a>) {
    self.push_try_scope();

    self.exec_block_statement(&node.block);

    let thrown_val = self.pop_try_scope().thrown_val();

    if let Some(handler) = &node.handler {
      self.exec_catch_clause(handler, thrown_val);
    }

    if let Some(finalizer) = &node.finalizer {
      self.exec_block_statement(finalizer);
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_try_statement(&self, node: &'a TryStatement<'a>) -> Option<Statement<'a>> {
    let TryStatement { span, block, handler, finalizer } = node;

    let block = self.transform_block_statement(block);

    let handler = if block.is_some() {
      handler.as_ref().map(|handler| self.transform_catch_clause(handler))
    } else {
      None
    };

    let finalizer =
      finalizer.as_ref().and_then(|finalizer| self.transform_block_statement(finalizer));

    match (block, finalizer) {
      (None, None) => None,
      (None, Some(finalizer)) => Some(self.ast_builder.statement_from_block(finalizer)),
      (Some(block), finalizer) => {
        Some(self.ast_builder.statement_try(*span, block, handler, finalizer))
      }
    }
  }
}
