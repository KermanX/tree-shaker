use crate::{
  analyzer::Analyzer,
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
    let properties = object.enumerate_properties(self, dep.into());

    let rest = self.new_empty_object();
    for (definite, key, value) in properties {
      rest.init_property(self, PropertyKind::Init, key, value, definite);
    }

    for key in enumerated {
      rest.delete_property(self, ().into(), key);
    }

    self.factory.new_entity(rest)
  }
}
