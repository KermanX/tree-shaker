use super::{react::create_react_namespace, Builtins};
use crate::config::TreeShakeJsxPreset;

impl<'a> Builtins<'a> {
  pub fn init_globals(&mut self) {
    let factory = self.factory;
    let globals = &mut self.globals;

    globals.insert("undefined", factory.undefined);
    globals.insert("Infinity", factory.infinity(true));
    globals.insert("NaN", factory.nan);
    globals.insert("undefined", factory.undefined);
    globals.insert("eval", factory.immutable_unknown);

    if self.config.jsx == TreeShakeJsxPreset::React {
      globals.insert("React", create_react_namespace(factory, self.prototypes));
    }
  }
}
