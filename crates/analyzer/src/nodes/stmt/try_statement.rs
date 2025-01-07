use crate::{host::Host, analyzer::Analyzer, scoping::CfScopeKind};
use oxc::{
  ast::ast::{Statement, TryStatement},
  span::SPAN,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_try_statement(&mut self, node: &'a TryStatement<'a>) {
    let labels = self.take_labels();
    self.push_cf_scope(CfScopeKind::Labeled, labels, Some(false));

    self.push_try_scope();
    self.exec_block_statement(&node.block);
    let try_scope = self.pop_try_scope();

    let uncaught = if let Some(handler) = &node.handler {
      self.exec_catch_clause(
        handler,
        // Theoretically, if `thrown_val` is `None`, it means that the `try` block
        // does not throw any value, so we should skip the `catch` block.
        // However, we can guarantee that all possible exceptions tracked.
        // For example, KeyboardInterrupt, which is not tracked, can be thrown.
        try_scope.thrown_val(self).unwrap_or_else(|| self.factory.unknown()),
      );
      None
    } else {
      try_scope.may_throw.then(|| try_scope.thrown_values.clone())
    };

    if let Some(finalizer) = &node.finalizer {
      self.exec_block_statement(finalizer);
    }

    if !self.cf_scope().must_exited() {
      if let Some(uncaught) = uncaught {
        self.forward_throw(uncaught.clone());
      }
    }

    self.pop_cf_scope();
  }
}

