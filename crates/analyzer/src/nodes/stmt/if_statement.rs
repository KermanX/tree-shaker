use crate::{
  analyzer::Analyzer,
  host::{EntityHost, Host},
  scoping::CfScopeKind,
};
use oxc::ast::ast::IfStatement;

#[allow(unused_variables)]
pub trait TraverseIfStatement<'a>: EntityHost<'a> {
  type Context;
  fn before_if_statement(&self, node: &'a IfStatement<'a>) -> Self::Context;
  fn after_if_test(
    &self,
    context: &mut Self::Context,
    test_value: Self::Entity,
    test_truthy: Option<bool>,
  ) {
  }
  fn before_if_branch(&self, context: &mut Self::Context, is_consequent: bool) {}
  fn after_if_branch(&self, context: &mut Self::Context, is_consequent: bool) {}
}

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_if_statement(&mut self, node: &'a IfStatement) {
    let labels = self.take_labels();

    let mut context = self.host.before_if_statement(node);

    let test_value = self.host.to_boolean(self.exec_expression(&node.test));
    let test_truthy = self.host.test_truthy(test_value);

    self.host.after_if_test(&mut context, test_value, test_truthy);

    let (maybe_consequent, maybe_alternate) = match test_truthy {
      Some(true) => (true, false),
      Some(false) => (false, true),
      None => (true, true),
    };

    let mut both_exit = true;
    let mut exit_target_inner = 0;
    let mut exit_target_outer = self.scope_context.cf.stack.len();
    let mut acc_dep_1 = None;
    let mut acc_dep_2 = None;

    if maybe_consequent {
      self.host.before_if_branch(&mut context, true);

      self.push_cf_scope(CfScopeKind::Labeled, labels.clone(), Some(false));
      self.init_statement(&node.consequent);
      self.pop_cf_scope();
      let conditional_scope = self.pop_cf_scope_and_get_mut();
      if let Some(stopped_exit) = conditional_scope.blocked_exit {
        exit_target_inner = exit_target_inner.max(stopped_exit);
        exit_target_outer = exit_target_outer.min(stopped_exit);
      } else {
        both_exit = false;
      }
      acc_dep_1 = conditional_scope.deps.try_collect(factory);
    }
    if maybe_alternate {
      self.host.before_if_branch(&mut context, false);

      if let Some(alternate) = &node.alternate {
        self.push_cf_scope(CfScopeKind::Labeled, labels.clone(), Some(false));
        self.init_statement(alternate);
        self.pop_cf_scope();
        let conditional_scope = self.pop_cf_scope_and_get_mut();
        if let Some(stopped_exit) = conditional_scope.blocked_exit {
          exit_target_inner = exit_target_inner.max(stopped_exit);
          exit_target_outer = exit_target_outer.min(stopped_exit);
        } else {
          both_exit = false;
        }
        acc_dep_2 = conditional_scope.deps.try_collect(factory);
      } else {
        self.pop_cf_scope();
        both_exit = false;
      }
    }

    let acc_dep = Some(self.consumable((acc_dep_1, acc_dep_2)));
    if both_exit {
      if let Some(acc_dep) =
        self.exit_to_impl(exit_target_inner, self.scope_context.cf.stack.len(), true, acc_dep)
      {
        self.exit_to_impl(exit_target_outer, exit_target_inner, false, acc_dep);
      }
    } else {
      self.exit_to_impl(exit_target_outer, self.scope_context.cf.stack.len(), false, acc_dep);
    }
  }
}
