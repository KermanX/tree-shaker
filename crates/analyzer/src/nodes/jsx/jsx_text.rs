use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::{Expression, JSXChild, JSXText};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_text(&mut self, _node: &'a JSXText<'a>) -> H::Entity {
    self.factory.immutable_unknown
  }
}
