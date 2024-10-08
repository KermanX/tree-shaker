use crate::{
  analyzer::Analyzer, ast::AstType2, data::StatementVecData, scope::CfScopeKind,
  transformer::Transformer,
};
use oxc::{
  ast::ast::{Expression, Statement, SwitchCase, SwitchStatement},
  span::Span,
};
use rustc_hash::FxHashSet;

const AST_TYPE: AstType2 = AstType2::SwitchStatement;

#[derive(Debug, Default)]
pub struct Data {
  need_test: FxHashSet<usize>,
  need_consequent: FxHashSet<usize>,
}

impl<'a> Analyzer<'a> {
  pub fn exec_switch_statement(&mut self, node: &'a SwitchStatement<'a>) {
    let labels = self.take_labels();
    let data = self.load_data::<Data>(AST_TYPE, node);

    // 1. discriminant
    let discriminant = self.exec_expression(&node.discriminant);
    self.push_cf_scope_for_deps(vec![discriminant.clone().into()]);

    // 2. tests
    let mut default_case = None;
    let mut maybe_default_case = Some(true);
    let mut test_results = vec![];
    let mut indeterminate = false;
    for (index, case) in node.cases.iter().enumerate() {
      if let Some(test) = &case.test {
        let test_val = self.exec_expression(test);

        let test_result = self.entity_op.strict_eq(self, discriminant, test_val);
        test_results.push(test_result);

        if test_result != Some(false) {
          data.need_test.insert(index);
        }

        match test_result {
          Some(true) => {
            maybe_default_case = Some(false);
            break;
          }
          Some(false) => {}
          None => {
            discriminant.consume(self);
            test_val.consume(self);
            // data.need_test.insert(index);
            maybe_default_case = None;
            if !indeterminate {
              indeterminate = true;
              self.push_cf_scope_normal(None);
            }
          }
        }
      } else {
        default_case = Some(index);
        test_results.push(/* Updated later */ None);
      }
    }
    if indeterminate {
      self.pop_cf_scope();
    }

    // Patch default case
    if let Some(default_case) = default_case {
      test_results[default_case] = maybe_default_case;
    }

    // 3. consequent
    self.push_cf_scope(CfScopeKind::BreakableWithoutLabel, labels, Some(false));
    let mut entered = Some(false);
    for (index, case) in node.cases.iter().enumerate() {
      if self.cf_scope().must_exited() {
        break;
      }

      let test_result = test_results.get(index).unwrap_or(&Some(false));

      entered = match (test_result, entered) {
        (Some(true), Some(true)) => unreachable!(),
        (Some(true), _) => Some(true),
        (Some(false), prev) => prev,
        (None, Some(true)) => Some(true),
        (None, _) => None,
      };

      if entered != Some(false) {
        data.need_consequent.insert(index);
      }

      let data = self.load_data::<StatementVecData>(AstType2::SwitchCase, case);

      self.push_cf_scope_normal(entered.map(|entered| !entered));
      self.exec_statement_vec(data, &case.consequent);
      self.pop_cf_scope();
    }

    self.pop_cf_scope();
    self.pop_cf_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_switch_statement(&self, node: &'a SwitchStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let SwitchStatement { span, discriminant, cases, .. } = node;

    let mut transformed_cases: Vec<(
      Span,
      Option<Expression<'a>>,
      oxc::allocator::Vec<'a, Statement<'a>>,
    )> = vec![];
    for (index, case) in cases.into_iter().enumerate() {
      let need_test = data.need_test.contains(&index);
      let need_consequent = data.need_consequent.contains(&index);

      let data = self.get_data::<StatementVecData>(AstType2::SwitchCase, case);

      let SwitchCase { test, consequent, .. } = case;

      let test = test.as_ref().map(|test| {
        if need_test || self.transform_expression(test, false).is_some() {
          self.transform_expression(test, true)
        } else {
          None
        }
      });

      let consequent = if need_consequent {
        self.transform_statement_vec(data, consequent)
      } else {
        self.ast_builder.vec()
      };

      match test {
        Some(None) => {
          if consequent.len() > 0 {
            if let Some(last) = transformed_cases.last_mut() {
              last.2.extend(consequent);
            } else {
              // In case the first case is default + no consequent
              transformed_cases.push((*span, None, consequent));
            }
          }
        }
        Some(Some(test)) => {
          transformed_cases.push((*span, Some(test), consequent));
        }
        None => {
          if consequent.len() > 0 {
            transformed_cases.push((*span, None, consequent));
          }
        }
      }
    }

    if transformed_cases.is_empty() {
      self
        .transform_expression(discriminant, false)
        .map(|expr| self.ast_builder.statement_expression(*span, expr))
    } else {
      let discriminant = self.transform_expression(discriminant, true).unwrap();

      Some(self.ast_builder.statement_switch(*span, discriminant, {
        let mut cases = self.ast_builder.vec();
        for (span, test, consequent) in transformed_cases {
          cases.push(self.ast_builder.switch_case(span, test, consequent));
        }
        cases
      }))
    }
  }
}
