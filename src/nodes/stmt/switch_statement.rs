use crate::{analyzer::Analyzer, ast::AstType2, transformer::Transformer};
use oxc::ast::ast::{Statement, SwitchCase, SwitchStatement};
use rustc_hash::FxHashSet;

use super::statement_vec::StatementVecData;

const AST_TYPE: AstType2 = AstType2::SwitchStatement;

#[derive(Debug, Default)]
pub struct Data {
  // need_test: FxHashSet<usize>,
  need_consequent: FxHashSet<usize>,
}

impl<'a> Analyzer<'a> {
  pub fn exec_switch_statement(&mut self, node: &'a SwitchStatement<'a>) {
    let data = self.load_data::<Data>(AST_TYPE, node);

    // 1. discriminant
    let discriminant = self.exec_expression(&node.discriminant);

    // 2. tests
    let mut default_case = None;
    let mut maybe_default_case = Some(true);
    let mut test_results = vec![];
    let mut indeterminate = false;
    for (index, case) in node.cases.iter().enumerate() {
      if let Some(test) = &case.test {
        let test_val = self.exec_expression(test);

        let test_result = self.entity_op.strict_eq(&discriminant, &test_val);
        test_results.push(test_result);

        match test_result {
          Some(true) => {
            maybe_default_case = Some(false);
            break;
          }
          Some(false) => {}
          None => {
            // data.need_test.insert(index);
            maybe_default_case = None;
            if !indeterminate {
              indeterminate = true;
              self.push_cf_scope(None, false);
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
    self.push_cf_scope(Some(false), false);
    let mut entered: Option<_> = Some(false);
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
      self.exec_statement_vec(data, entered.map(|entered| !entered), &case.consequent);
    }

    self.pop_cf_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_switch_statement(&mut self, node: SwitchStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let SwitchStatement { span, discriminant, cases, .. } = node;

    let discriminant = self.transform_expression(discriminant, true).unwrap();

    let mut transformed_cases = self.ast_builder.vec();
    for (index, case) in cases.into_iter().enumerate() {
      let need_consequent = data.need_consequent.contains(&index);
      let data = self.get_data::<StatementVecData>(AstType2::SwitchCase, &case);

      let SwitchCase { test, consequent, .. } = case;

      // TODO: tree shake if test is readonly
      let test = test.map(|test| self.transform_expression(test, true).unwrap());

      let consequent = if need_consequent {
        self.transform_statement_vec(data, consequent)
      } else {
        self.ast_builder.vec()
      };

      if test.is_some() || !consequent.is_empty() {
        transformed_cases.push(self.ast_builder.switch_case(span, test, consequent));
      }
    }

    Some(self.ast_builder.statement_switch(span, discriminant, transformed_cases))
  }
}
