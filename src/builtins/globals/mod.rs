use super::Builtins;
use crate::init_map;

impl<'a> Builtins<'a> {
  pub fn init_globals(&mut self) {
    let factory = self.factory;
    let globals = &mut self.globals;

    init_map!(globals, {
      "undefined" => factory.undefined,
      "Infinity" => factory.infinity(true),
      "NaN" => factory.nan,
      "eval" => factory.immutable_unknown,
    })
  }
}
