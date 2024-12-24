use crate::{
  analyzer::Analyzer, dep::DepId, entity::Entity, mangling::MangleAtom, transformer::Transformer,
};
use oxc::span::GetSpan;

impl<'a> Analyzer<'a> {
  pub fn exec_mangable_static_string(&mut self, key: impl Into<DepId>, str: &'a str) -> Entity<'a> {
    let key = key.into();
    let atom = *self.get_data_or_insert_with(key, || MangleAtom::new(key.span()));
    self.factory.mangable_string(str, atom)
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_mangable_static_string(
    &self,
    key: impl Into<DepId>,
    original: &'a str,
  ) -> &'a str {
    let atom = *self.force_get_data::<MangleAtom>(key.into());
    let mut mangler = self.mangler.borrow_mut();
    mangler.resolve(atom).unwrap_or(original)
  }
}
