use std::usize;

use crate::{analyzer::Analyzer, ast::AstType2, scope::CfScopeKind, transformer::Transformer};
use oxc::{
  ast::ast::{IfStatement, Statement},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::IfStatement;

#[derive(Debug, Default, Clone)]
pub struct Data {
  maybe_true: bool,
  maybe_false: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_if_statement(&mut self, node: &'a IfStatement) {
    let labels = self.take_labels();

    let test = self.exec_expression(&node.test);

    let (maybe_true, maybe_false) = match test.test_truthy() {
      Some(true) => (true, false),
      Some(false) => (false, true),
      None => (true, true),
    };

    let indeterminate = maybe_true && maybe_false;

    if indeterminate {
      test.consume_self(self);
    }

    let branch_exited = if indeterminate { None } else { Some(false) };
    let mut should_exit = true;
    let mut exit_target_inner = 0;
    let mut exit_target_outer = usize::MAX;

    if maybe_true {
      self.push_cf_scope(CfScopeKind::If, None, branch_exited);
      self.push_cf_scope(CfScopeKind::Normal, labels.clone(), Some(false));
      self.exec_statement(&node.consequent);
      self.pop_cf_scope();
      if let Some(stopped_exit) = self.pop_cf_scope().borrow().stopped_exit {
        exit_target_inner = exit_target_inner.max(stopped_exit);
        exit_target_outer = exit_target_outer.min(stopped_exit);
      } else {
        should_exit = false;
      }
    }
    if maybe_false {
      if let Some(alternate) = &node.alternate {
        self.push_cf_scope(CfScopeKind::If, None, branch_exited);
        self.push_cf_scope(CfScopeKind::Normal, labels.clone(), Some(false));
        self.exec_statement(alternate);
        self.pop_cf_scope();
        if let Some(stopped_exit) = self.pop_cf_scope().borrow().stopped_exit {
          exit_target_inner = exit_target_inner.max(stopped_exit);
          exit_target_outer = exit_target_outer.min(stopped_exit);
        } else {
          should_exit = false;
        }
      } else {
        should_exit = false;
      }
    }

    if should_exit {
      self.exit_to(exit_target_inner);
      for cf_scope in self.scope_context.cf_scopes[exit_target_outer..exit_target_inner].iter_mut()
      {
        cf_scope.borrow_mut().exited = None;
      }
    }

    let data = self.load_data::<Data>(AST_TYPE, node);

    data.maybe_true |= maybe_true;
    data.maybe_false |= maybe_false;
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

    let need_test_val =
      data.maybe_true && data.maybe_false && (consequent.is_some() || alternate.is_some());
    let test = self.transform_expression(test, need_test_val);

    let mut statements = self.ast_builder.vec();

    match (data.maybe_true, data.maybe_false) {
      (true, true) => {
        // Both cases are possible
        return match (consequent, alternate) {
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
        };
      }
      (true, false) => {
        // Only one case is possible
        test.map(|test| statements.push(self.ast_builder.statement_expression(test.span(), test)));
        consequent.map(|body| statements.push(body));
      }
      (false, true) => {
        // Only one case is possible
        test.map(|test| statements.push(self.ast_builder.statement_expression(test.span(), test)));
        alternate.map(|body| statements.push(body));
      }
      (false, false) => unreachable!(),
    };

    if statements.is_empty() {
      None
    } else {
      Some(self.ast_builder.statement_block(*span, statements))
    }
  }
}
