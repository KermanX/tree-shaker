use crate::{
  analyzer::Analyzer,
  consumable::box_consumable,
  entity::{Entity, EntityDepNode, EntityTrait},
};

impl<'a> Analyzer<'a> {
  pub fn exec_object_rest(
    &mut self,
    dep: impl Into<EntityDepNode>,
    object: Entity<'a>,
    enumerated: Vec<Entity<'a>>,
  ) -> Entity<'a> {
    let rest = self.new_empty_object();
    rest.init_spread(self, box_consumable(dep.into()), object);
    for key in enumerated {
      rest.delete_property(self, box_consumable(()), key);
    }

    self.factory.new_entity(rest)
  }
}
