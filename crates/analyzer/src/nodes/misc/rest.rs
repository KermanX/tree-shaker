use crate::{host::Host, analyzer::Analyzer, dep::DepId};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  /// const { enumerated_1, enumerated_2, ...rest } = object;
  pub fn exec_object_rest(
    &mut self,
    dep: impl Into<DepId>,
    object: H::Entity,
    enumerated: Vec<H::Entity>,
  ) -> H::Entity {
    let rest = self.new_empty_object(&self.builtins.prototypes.object, None);
    rest.init_spread(self, self.consumable(dep.into()), object);
    for key in enumerated {
      rest.delete_property(self, self.factory.empty_consumable, key);
    }

    rest
  }
}
