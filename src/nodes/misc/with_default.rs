use crate::{
  analyzer::Analyzer,
  entity::{entity::Entity, union::UnionEntity},
};
use oxc::ast::ast::Expression;

impl<'a> Analyzer<'a> {
  pub fn exec_with_default(
    &mut self,
    default: &'a Expression<'a>,
    value: Entity<'a>,
  ) -> (bool, Entity<'a>) {
    let is_undefined = value.test_is_undefined();
    let binding_val = match is_undefined {
      Some(true) => self.exec_expression(default),
      Some(false) => value,
      None => {
        self.push_cf_scope_normal(None);
        let value = UnionEntity::new(vec![self.exec_expression(default), value]);
        self.pop_cf_scope();
        value
      }
    };
    (is_undefined != Some(false), binding_val)
  }
}
