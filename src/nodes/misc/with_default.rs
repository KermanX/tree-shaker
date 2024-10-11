use crate::{analyzer::Analyzer, entity::Entity};
use oxc::ast::ast::Expression;

impl<'a> Analyzer<'a> {
  pub fn exec_with_default(
    &mut self,
    default: &'a Expression<'a>,
    value: Entity<'a>,
  ) -> (bool, Entity<'a>) {
    let is_undefined = value.test_is_undefined();

    self.push_cf_scope_for_dep(value.to_consumable());
    let binding_val = match is_undefined {
      Some(true) => {
        let default_val = self.exec_expression(default);
        self.factory.new_computed(default_val, value.to_consumable())
      }
      Some(false) => value,
      None => {
        self.push_cf_scope_normal(None);
        let default_val = self.exec_expression(default);
        let value = self.factory.new_union(vec![default_val, value]);
        self.pop_cf_scope();
        value
      }
    };
    self.pop_cf_scope();

    (is_undefined != Some(false), binding_val)
  }
}
