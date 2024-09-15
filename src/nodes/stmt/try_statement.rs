use crate::{
  analyzer::Analyzer, entity::unknown::UnknownEntity, scope::CfScopeKind, transformer::Transformer,
};
use oxc::ast::ast::{Statement, TryStatement};

impl<'a> Analyzer<'a> {
  pub fn exec_try_statement(&mut self, node: &'a TryStatement<'a>) {
    let labels = self.take_labels();
    self.push_cf_scope(CfScopeKind::Normal, labels, Some(false));

    self.push_try_scope();
    self.exec_block_statement(&node.block);
    let thrown_val = self.pop_try_scope().thrown_val();

    if let Some(handler) = &node.handler {
      self.exec_catch_clause(
        handler,
        // Theoretically, if `thrown_val` is `None`, it means that the `try` block
        // does not throw any value, so we should skip the `catch` block.
        // However, we can guarantee that all possible exceptions tracked.
        // For example, KeyboardInterrupt, which is not tracked, can be thrown.
        thrown_val.unwrap_or_else(|| UnknownEntity::new_unknown()),
      );
    }

    if let Some(finalizer) = &node.finalizer {
      self.exec_block_statement(finalizer);
    }

    self.pop_cf_scope();
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
