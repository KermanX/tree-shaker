use crate::{analyzer::Analyzer, dep::DepId, entity::Entity};

impl<'a> Analyzer<'a> {
  /// const { enumerated_1, enumerated_2, ...rest } = object;
  pub fn exec_object_rest(
    &mut self,
    dep: impl Into<DepId>,
    object: Entity<'a>,
    enumerated: Vec<Entity<'a>>,
  ) -> Entity<'a> {
    let rest = self.new_empty_object(&self.builtins.prototypes.object, None);
    rest.init_spread(self, self.consumable(dep.into()), object);
    for key in enumerated {
      rest.delete_property(self, self.factory.empty_consumable, key);
    }

    rest
  }
}
