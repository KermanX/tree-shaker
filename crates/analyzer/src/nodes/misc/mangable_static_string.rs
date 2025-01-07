use crate::{host::Host, 
  analyzer::Analyzer, dep::DepId, mangling::MangleAtom,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_mangable_static_string(&mut self, key: impl Into<DepId>, str: &'a str) -> H::Entity {
    let atom = self.load_data::<Option<MangleAtom>>(key.into());
    self.factory.mangable_string(str, *atom.get_or_insert_with(|| self.mangler.new_atom()))
  }
}

