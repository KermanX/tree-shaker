use crate::{analyzer::Analyzer, entity::Entity};
use oxc::ast::ast::JSXAttributeName;

impl<'a> Analyzer<'a> {
  pub fn exec_jsx_attribute_name(&mut self, node: &'a JSXAttributeName<'a>) -> Entity<'a> {
    self.factory.string(match node {
      JSXAttributeName::Identifier(node) => node.name.as_str(),
      JSXAttributeName::NamespacedName(node) => self.allocator.alloc(format!(
        "{}:{}",
        node.namespace.name.as_str(),
        node.property.name.as_str()
      )),
    })
  }
}
