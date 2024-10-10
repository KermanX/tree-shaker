use crate::{
  analyzer::Analyzer,
  consumable::box_consumable,
  entity::{Entity, EntityDepNode, EntityTrait},
};
use oxc::ast::ast::PropertyKind;

impl<'a> Analyzer<'a> {
  pub fn exec_object_rest(
    &mut self,
    dep: impl Into<EntityDepNode>,
    object: Entity<'a>,
    enumerated: Vec<Entity<'a>>,
  ) -> Entity<'a> {
    let properties = object.enumerate_properties(self, box_consumable(dep.into()));

    let rest = self.new_empty_object();
    for (definite, key, value) in properties {
      rest.init_property(PropertyKind::Init, key, value, definite);
    }

    for key in enumerated {
      rest.delete_property(self, box_consumable(()), &key);
    }

    Entity::new(rest)
  }
}
