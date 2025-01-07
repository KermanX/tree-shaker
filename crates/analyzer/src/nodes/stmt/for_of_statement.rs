use crate::{host::Host, analyzer::Analyzer,  scoping::CfScopeKind};
use oxc::{
  ast::ast::{ForOfStatement, Statement},
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_for_of_statement(&mut self, node: &'a ForOfStatement<'a>) {
    let labels = self.take_labels();

    let right = self.exec_expression(&node.right);
    let right = if node.r#await {
      right.consume(self);
      self.refer_dep(AstKind2::ForOfStatement(node));
      self.factory.immutable_unknown
    } else {
      right
    };

    self.declare_for_statement_left(&node.left);

    let Some(iterated) =
      right.iterate_result_union(self, self.consumable(AstKind2::ForOfStatement(node)))
    else {
      return;
    };

    let dep = self.consumable((AstKind2::ForOfStatement(node), right));

    self.push_cf_scope_with_deps(
      CfScopeKind::BreakableWithoutLabel,
      labels.clone(),
      vec![dep],
      Some(false),
    );
    self.exec_loop(move |analyzer| {
      analyzer.declare_for_statement_left(&node.left);
      analyzer.init_for_statement_left(&node.left, iterated);

      analyzer.push_cf_scope(CfScopeKind::Continuable, labels.clone(), None);
      analyzer.init_statement(&node.body);
      analyzer.pop_cf_scope();
    });
    self.pop_cf_scope();
  }
}

