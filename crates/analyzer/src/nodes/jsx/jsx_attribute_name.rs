use crate::{analyzer::Analyzer, host::Host};
use oxc::{allocator::Allocator, ast::ast::JSXAttributeName, span::GetSpan};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_attribute_name(&mut self, node: &'a JSXAttributeName<'a>) -> H::Entity {
    self
      .exec_mangable_static_string(AstKind2::JSXAttributeName(node), get_text(self.allocator, node))
  }
}
