use crate::{analyzer::Analyzer, ast::AstType2, scope::CfScopeKind, transformer::Transformer};
use oxc::{
  ast::{
    ast::{IfStatement, Statement},
    AstKind,
  },
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::IfStatement;

#[derive(Debug, Default)]
pub struct Data {
  maybe_true: bool,
  maybe_false: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_if_statement(&mut self, node: &'a IfStatement) {
    let labels = self.take_labels();

    let test = self.exec_expression(&node.test).get_to_boolean(self);

    let (maybe_true, maybe_false) = match test.test_truthy() {
      Some(true) => (true, false),
      Some(false) => (false, true),
      None => (true, true),
    };

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.maybe_true |= maybe_true;
    data.maybe_false |= maybe_false;

    let mut should_exit = true;
    let mut exit_target_inner = 0;
    let mut exit_target_outer = self.scope_context.cf.stack.len();
    let mut acc_dep = None;

    if maybe_true {
      self.push_conditional_cf_scope(
        AstKind::IfStatement(node),
        CfScopeKind::IfStatement,
        test.clone(),
        maybe_true,
        maybe_false,
      );
      self.push_cf_scope(CfScopeKind::Normal, labels.clone(), Some(false));
      self.exec_statement(&node.consequent);
      self.pop_cf_scope();
      let conditional_scope = self.pop_cf_scope_and_get_mut();
      if let Some(stopped_exit) = conditional_scope.blocked_exit {
        exit_target_inner = exit_target_inner.max(stopped_exit);
        exit_target_outer = exit_target_outer.min(stopped_exit);
      } else {
        should_exit = false;
      }
      acc_dep.get_or_insert_with(|| conditional_scope.deps.collect());
    }
    if maybe_false {
      if let Some(alternate) = &node.alternate {
        self.push_conditional_cf_scope(
          AstKind::IfStatement(node),
          CfScopeKind::IfStatement,
          test,
          maybe_true,
          maybe_false,
        );
        self.push_cf_scope(CfScopeKind::Normal, labels.clone(), Some(false));
        self.exec_statement(alternate);
        self.pop_cf_scope();
        let conditional_scope = self.pop_cf_scope_and_get_mut();
        if let Some(stopped_exit) = conditional_scope.blocked_exit {
          exit_target_inner = exit_target_inner.max(stopped_exit);
          exit_target_outer = exit_target_outer.min(stopped_exit);
        } else {
          should_exit = false;
        }
        acc_dep.get_or_insert_with(|| conditional_scope.deps.collect());
      } else {
        should_exit = false;
      }
    }

    if should_exit {
      let acc_dep =
        self.exit_to_impl(exit_target_inner, self.scope_context.cf.stack.len(), true, acc_dep);
      self.exit_to_impl(exit_target_outer, exit_target_inner, false, acc_dep);
    } else {
      self.exit_to_impl(exit_target_outer, self.scope_context.cf.stack.len(), false, acc_dep);
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_if_statement(&self, node: &'a IfStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let IfStatement { span, test, consequent, alternate, .. } = node;

    let consequent = if data.maybe_true { self.transform_statement(consequent) } else { None };
    let alternate = if data.maybe_false {
      alternate.as_ref().and_then(|alt| self.transform_statement(alt))
    } else {
      None
    };

    let need_test_val = self.is_referred(AstKind::IfStatement(node));

    let test = self.transform_expression(test, need_test_val);

    if need_test_val {
      match (consequent, alternate) {
        (Some(consequent), alternate) => {
          Some(self.ast_builder.statement_if(*span, test.unwrap(), consequent, alternate))
        }
        (None, Some(alternate)) => Some(self.ast_builder.statement_if(
          *span,
          self.build_negate_expression(test.unwrap()),
          alternate,
          None,
        )),
        (None, None) => test.map(|test| self.ast_builder.statement_expression(test.span(), test)),
      }
    } else {
      let mut statements = self.ast_builder.vec();
      if let Some(test) = test {
        statements.push(self.ast_builder.statement_expression(test.span(), test));
      }
      if let Some(consequent) = consequent {
        statements.push(consequent);
      }
      if let Some(alternate) = alternate {
        statements.push(alternate);
      }

      if statements.is_empty() {
        None
      } else {
        Some(self.ast_builder.statement_block(*span, statements))
      }
    }
  }
}
