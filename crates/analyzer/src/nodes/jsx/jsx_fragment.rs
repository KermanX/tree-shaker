use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  allocator,
  ast::ast::{Expression, JSXFragment},
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_fragment(&mut self, node: &'a JSXFragment<'a>) -> H::Entity {
    // already computed unknown
    self.exec_jsx_children(&node.children)
  }
}
