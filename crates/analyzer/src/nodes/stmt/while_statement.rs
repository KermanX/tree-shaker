use crate::{analyzer::Analyzer, host::Host, scoping::CfScopeKind};
use oxc::{
  ast::ast::{Statement, WhileStatement},
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_while_statement(&mut self, node: &'a WhileStatement<'a>) {
    let labels = self.take_labels();

    // This may be indeterminate. However, we can't know it until we execute the test.
    // And there should be no same level break/continue statement in test.
    // `a: while(() => { break a }) { }` is illegal.
    let test = self.exec_expression(&node.test);

    if test.test_truthy() == Some(false) {
      return;
    }

    let dep = self.consumable((AstKind2::WhileStatement(node), test));

    self.push_cf_scope_with_deps(
      CfScopeKind::BreakableWithoutLabel,
      labels.clone(),
      vec![dep],
      Some(false),
    );
    self.exec_loop(move |analyzer| {
      analyzer.push_cf_scope(CfScopeKind::Continuable, labels.clone(), None);

      analyzer.init_statement(&node.body);
      analyzer.exec_expression(&node.test).consume(analyzer);

      analyzer.pop_cf_scope();

      let test = analyzer.exec_expression(&node.test);
      let test = analyzer.consumable(test);
      analyzer.cf_scope_mut().push_dep(test);
    });
    self.pop_cf_scope();
  }
}
