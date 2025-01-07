use crate::{host::Host, analyzer::Analyzer, ast::DeclarationKind};
use oxc::{
  ast::ast::{CatchClause, CatchParameter},
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_catch_clause(&mut self, node: &'a CatchClause<'a>, value: H::Entity) {
    self.push_indeterminate_cf_scope();

    if let Some(param) = &node.param {
      self.declare_binding_pattern(&param.pattern, false, DeclarationKind::Caught);
      self.init_binding_pattern(&param.pattern, Some(value));
    }

    self.exec_block_statement(&node.body);

    self.pop_cf_scope();
  }
}

