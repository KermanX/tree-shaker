use super::Builtins;
use crate::{config::TreeShakeJsxPreset, entity::Entity};

#[derive(Debug, Clone, Copy)]
pub struct KnownModule<'a> {
  pub namespace: Entity<'a>,
  pub default: Entity<'a>,
}

impl<'a> Builtins<'a> {
  pub fn init_known_modules(&mut self) {
    let known_modules = &mut self.known_modules;

    if self.config.jsx == TreeShakeJsxPreset::React {
      known_modules.insert("react", {
        let value = *self.globals.get("React").unwrap();
        KnownModule { namespace: value, default: value }
      });
    }
  }
}
