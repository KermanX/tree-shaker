use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  allocator,
  ast::{
    ast::{Expression, JSXClosingElement, JSXElement, JSXOpeningElement, PropertyKind},
    NONE,
  },
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_element(&mut self, node: &'a JSXElement<'a>) -> H::Entity {
    let tag = self.exec_jsx_element_name(&node.opening_element.name);
    let attributes = self.exec_jsx_attributes(&node.opening_element);
    let children = self.exec_jsx_children(&node.children);
    let key_children = *self.builtins.react_data.key_children.get_or_insert_with(|| {
      self.factory.mangable_string("children", self.mangler.new_constant_atom("children"))
    });
    attributes.init_property(self, PropertyKind::Init, key_children, children, true);
    self.factory.react_element(tag, attributes)
  }
}
