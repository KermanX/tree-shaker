use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::CallExpression;



impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_call_expression(&mut self, node: &'a CallExpression) -> H::Entity {
    let (scope_count, value, undefined) = self.exec_call_expression_in_chain(node).unwrap();

    assert_eq!(scope_count, 0);
    assert!(undefined.is_none());

    value
  }

  /// Returns (short-circuit, value)
  pub fn exec_call_expression_in_chain(
    &mut self,
    node: &'a CallExpression,
  ) -> Result<(usize, H::Entity, Option<H::Entity>), H::Entity> {
    let (mut scope_count, callee, mut undefined, this) = self.exec_callee(&node.callee)?;

    if node.optional {
      let maybe_left = match callee.test_nullish() {
        Some(true) => {
          self.pop_multiple_cf_scopes(scope_count);
          return Err(self.forward_logical_left_val(dep_id, self.factory.undefined, true, false));
        }
        Some(false) => false,
        None => {
          undefined = Some(self.forward_logical_left_val(
            dep_id,
            undefined.unwrap_or(self.factory.undefined),
            true,
            false,
          ));
          true
        }
      };

      self.push_logical_right_cf_scope(dep_id, callee, maybe_left, true);
      scope_count += 1;
    }

    let args = self.exec_arguments(&node.arguments);

    let ret_val = self.host.call(node, callee, this, args);

    Ok((scope_count, ret_val, undefined))
  }
}
