use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::{Expression, Super};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_super(&mut self, _node: &'a Super) -> H::Entity {
    self.factory.unknown()
  }
}
