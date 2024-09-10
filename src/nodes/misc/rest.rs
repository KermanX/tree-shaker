use crate::{
  analyzer::Analyzer,
  entity::entity::{Entity, EntityTrait},
};
use oxc::ast::ast::PropertyKind;

impl<'a> Analyzer<'a> {
  pub fn exec_object_rest(
    &mut self,
    object: Entity<'a>,
    enumerated: Vec<Entity<'a>>,
  ) -> (bool, Entity<'a>) {
    let (has_effect, properties) = object.enumerate_properties(self);

    let rest = self.new_empty_object();
    for (definite, key, value) in properties {
      rest.init_property(PropertyKind::Init, key, value, definite);
    }

    for key in enumerated {
      rest.delete_property(self, &key);
    }

    (has_effect, Entity::new(rest))
  }
}
