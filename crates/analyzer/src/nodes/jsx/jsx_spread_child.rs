use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  allocator,
  ast::ast::{Expression, JSXSpreadChild},
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_spread_child(&mut self, _node: &'a JSXSpreadChild<'a>) -> H::Entity {
    self.factory.immutable_unknown
  }
}
