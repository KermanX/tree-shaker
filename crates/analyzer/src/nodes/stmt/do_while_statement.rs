use crate::{analyzer::Analyzer, host::Host, scoping::CfScopeKind};
use oxc::{
  ast::ast::{DoWhileStatement, NumberBase, Statement},
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_do_while_statement(&mut self, node: &'a DoWhileStatement<'a>) {
    let labels = self.take_labels();
    self.push_cf_scope(CfScopeKind::BreakableWithoutLabel, labels.clone(), Some(false));

    // Execute the first round.
    self.push_cf_scope(CfScopeKind::Continuable, labels.clone(), Some(false));
    self.init_statement(&node.body);
    self.pop_cf_scope();

    if self.cf_scope().must_exited() {
      self.pop_cf_scope();
      return;
    }

    data.need_test = true;
    let test = self.exec_expression(&node.test);

    // The rest is the same as while statement.
    if test.test_truthy() == Some(false) {
      self.pop_cf_scope();
      return;
    }
    test.consume(self);

    data.need_loop = true;

    self.exec_loop(move |analyzer| {
      analyzer.push_cf_scope(CfScopeKind::Continuable, labels.clone(), None);

      analyzer.init_statement(&node.body);
      analyzer.exec_expression(&node.test).consume(analyzer);

      analyzer.pop_cf_scope();
    });

    self.pop_cf_scope();
  }
}
