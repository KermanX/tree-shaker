use crate::{
  analyzer::Analyzer,
  entity::{ComputedEntity, Entity, UnionEntity},
};
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
      Some(true) => ComputedEntity::new(self.exec_expression(default), value.to_consumable()),
      Some(false) => value,
      None => {
        self.push_cf_scope_normal(None);
        let value = UnionEntity::new(vec![self.exec_expression(default), value]);
        self.pop_cf_scope();
        value
      }
    };
    self.pop_cf_scope();

    (is_undefined != Some(false), binding_val)
  }
}
