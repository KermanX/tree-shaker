use super::{
  react::{create_react_jsx_runtime_namespace, create_react_namespace},
  Builtins,
};
use crate::entity::Entity;

#[derive(Debug, Clone, Copy)]
pub struct KnownModule<'a> {
  pub namespace: Entity<'a>,
  pub default: Entity<'a>,
}

impl<'a> Builtins<'a> {
  pub fn init_known_modules(&mut self) {
    let known_modules = &mut self.known_modules;

    known_modules.insert("react", {
      let value = create_react_namespace(self.factory, self.prototypes);
      KnownModule { namespace: value, default: value }
    });
    known_modules.insert("react/jsx-runtime", {
      let value = create_react_jsx_runtime_namespace(self.factory, self.prototypes);
      KnownModule { namespace: value, default: value }
    });
  }

  pub fn get_known_module(&self, name: &str) -> Option<KnownModule<'a>> {
    let name = name.strip_prefix("https://esm.sh/").unwrap_or(name);
    self.known_modules.get(name).copied()
  }
}
