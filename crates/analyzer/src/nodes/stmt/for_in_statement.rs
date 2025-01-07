use crate::{
  analyzer::Analyzer,
  host::{Host, TypeofResult},
  scoping::CfScopeKind,
};
use oxc::ast::ast::ForInStatement;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_for_in_statement(&mut self, node: &'a ForInStatement<'a>) {
    let labels = self.take_labels();
    let right = self.exec_expression(&node.right);

    // FIXME: enumerate keys!
    right.consume(self);

    let types_have_no_keys: TypeofResult = TypeofResult::Undefined
      | TypeofResult::Boolean
      | TypeofResult::Number
      | TypeofResult::String
      | TypeofResult::Symbol;

    // TODO: empty object, simple function, array
    if (right.test_typeof() & !types_have_no_keys) == TypeofResult::_None
      || right.test_nullish() == Some(true)
    {
      return;
    }

    self.declare_for_statement_left(&node.left);

    let dep = self.consumable((AstKind2::ForInStatement(node), right));

    self.push_cf_scope_with_deps(
      CfScopeKind::BreakableWithoutLabel,
      labels.clone(),
      vec![dep],
      Some(false),
    );
    self.exec_loop(move |analyzer| {
      analyzer.declare_for_statement_left(&node.left);
      analyzer.init_for_statement_left(&node.left, analyzer.factory.unknown_string);

      analyzer.push_cf_scope(CfScopeKind::Continuable, labels.clone(), None);
      analyzer.init_statement(&node.body);
      analyzer.pop_cf_scope();
    });
    self.pop_cf_scope();
  }
}
