use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::ast::Expression;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_with_default(
    &mut self,
    default: &'a Expression<'a>,
    value: H::Entity,
  ) -> (bool, H::Entity) {
    let is_undefined = value.test_is_undefined();

    self.push_dependent_cf_scope(value);
    let binding_val = match is_undefined {
      Some(true) => {
        let default_val = self.exec_expression(default);
        self.factory.computed(default_val, value)
      }
      Some(false) => value,
      None => {
        self.push_indeterminate_cf_scope();
        let default_val = self.exec_expression(default);
        let value = self.factory.union((default_val, value));
        self.pop_cf_scope();
        value
      }
    };
    self.pop_cf_scope();

    (is_undefined != Some(false), binding_val)
  }
}
