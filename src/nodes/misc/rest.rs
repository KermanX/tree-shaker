use crate::{
  analyzer::Analyzer,
  consumable::box_consumable,
  dep::DepId,
  entity::{Entity, EntityTrait},
};

impl<'a> Analyzer<'a> {
  /// const { enumerated_1, enumerated_2, ...rest } = object;
  pub fn exec_object_rest(
    &mut self,
    dep: impl Into<DepId>,
    object: Entity<'a>,
    enumerated: Vec<Entity<'a>>,
  ) -> Entity<'a> {
    let rest = self.new_empty_object(&self.builtins.prototypes.object, None);
    rest.init_spread(self, box_consumable(dep.into()), object);
    for key in enumerated {
      rest.delete_property(self, box_consumable(()), key);
    }

    self.factory.entity(rest)
  }
}
