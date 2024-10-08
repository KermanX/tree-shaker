use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::Consumable,
  scope::{conditional::ConditionalData, CfScopeKind},
  transformer::Transformer,
};
use oxc::{
  ast::ast::{IfStatement, Statement},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::IfStatement;

#[derive(Debug, Default)]
pub struct Data<'a> {
  maybe_true: bool,
  maybe_false: bool,
  conditional: ConditionalData<'a>,
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

    let historical_indeterminate = data.maybe_true && data.maybe_false;
    let current_indeterminate = maybe_true && maybe_false;

    let mut should_exit = true;
    let mut exit_target_inner = 0;
    let mut exit_target_outer = self.scope_context.cf.stack.len();

    /*
     dep is AstKind::IfStatement

     Then,
     - If only one case is possible, collect `test` value
     - If both cases are possible (data.maybe_true && data.maybe_false),
       then, if dep is referred, consume all collected `test` values, then clear them.
    */

    let mut deps: Option<Vec<Consumable<'a>>> = None;

    if maybe_true {
      self.push_conditional_cf_scope(
        &mut data.conditional,
        CfScopeKind::IfStatement,
        test.clone(),
        historical_indeterminate,
        current_indeterminate,
      );
      self.push_cf_scope(CfScopeKind::Normal, labels.clone(), Some(false));
      self.exec_statement(&node.consequent);
      self.pop_cf_scope();
      let conditional_scope = self.pop_cf_scope_and_get();
      if let Some(stopped_exit) = conditional_scope.blocked_exit {
        exit_target_inner = exit_target_inner.max(stopped_exit);
        exit_target_outer = exit_target_outer.min(stopped_exit);
      } else {
        should_exit = false;
      }
      deps.get_or_insert_with(|| conditional_scope.deps.clone());
    }
    if maybe_false {
      if let Some(alternate) = &node.alternate {
        self.push_conditional_cf_scope(
          &mut data.conditional,
          CfScopeKind::IfStatement,
          test,
          historical_indeterminate,
          current_indeterminate,
        );
        self.push_cf_scope(CfScopeKind::Normal, labels.clone(), Some(false));
        self.exec_statement(alternate);
        self.pop_cf_scope();
        let conditional_scope = self.pop_cf_scope_and_get();
        if let Some(stopped_exit) = conditional_scope.blocked_exit {
          exit_target_inner = exit_target_inner.max(stopped_exit);
          exit_target_outer = exit_target_outer.min(stopped_exit);
        } else {
          should_exit = false;
        }
        deps.get_or_insert_with(|| conditional_scope.deps.clone());
      } else {
        should_exit = false;
      }
    }

    let deps = deps.unwrap_or_default();
    if should_exit {
      let deps =
        self.exit_to_impl(exit_target_inner, self.scope_context.cf.stack.len(), true, deps);
      self.exit_to_impl(exit_target_outer, exit_target_inner, false, deps);
    } else {
      self.exit_to_impl(exit_target_outer, self.scope_context.cf.stack.len(), false, deps);
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
