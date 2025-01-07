use crate::{host::Host, 
  analyzer::Analyzer,   scoping::CfScopeKind,
  };
use oxc::ast::ast::{ConditionalExpression, Expression, LogicalOperator};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_conditional_expression(&mut self, node: &'a ConditionalExpression<'a>) -> H::Entity {
    let test = self.exec_expression(&node.test);

    let (maybe_true, maybe_false) = match test.test_truthy() {
      Some(true) => (true, false),
      Some(false) => (false, true),
      None => (true, true),
    };

    let exec_consequent = move |analyzer: &mut Analyzer<'a>| {
      let conditional_dep = analyzer.push_if_like_branch_cf_scope(
        AstKind2::ConditionalExpression(node),
        CfScopeKind::ConditionalExprBranch,
        test,
        maybe_true,
        maybe_false,
        true,
        true,
      );
      let value = analyzer.exec_expression(&node.consequent);
      analyzer.pop_cf_scope();
      analyzer.factory.computed(value, conditional_dep)
    };

    let exec_alternate = move |analyzer: &mut Analyzer<'a>| {
      let conditional_dep = analyzer.push_if_like_branch_cf_scope(
        AstKind2::ConditionalExpression(node),
        CfScopeKind::ConditionalExprBranch,
        test,
        maybe_true,
        maybe_false,
        false,
        true,
      );
      let value = analyzer.exec_expression(&node.alternate);
      analyzer.pop_cf_scope();
      analyzer.factory.computed(value, conditional_dep)
    };

    match (maybe_true, maybe_false) {
      (true, false) => exec_consequent(self),
      (false, true) => exec_alternate(self),
      (true, true) => {
        let v1 = exec_consequent(self);
        let v2 = exec_alternate(self);
        self.factory.union((v1, v2))
      }
      _ => unreachable!("Conditional expression should have at least one possible branch"),
    }
  }
}

