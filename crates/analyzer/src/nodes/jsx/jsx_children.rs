use crate::{host::Host, analyzer::Analyzer};
use oxc::{
  allocator,
  ast::ast::{Expression, JSXChild},
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_children(&mut self, node: &'a allocator::Vec<'a, JSXChild<'a>>) -> H::Entity {
    let values: Vec<_> = node
      .iter()
      .map(|child| match child {
        JSXChild::Text(node) => self.exec_jsx_text(node),
        JSXChild::Element(node) => self.exec_jsx_element(node),
        JSXChild::Fragment(node) => self.exec_jsx_fragment(node),
        JSXChild::ExpressionContainer(node) => {
          self.exec_jsx_expression_container_as_jsx_child(node)
        }
        JSXChild::Spread(node) => self.exec_jsx_spread_child(node),
      })
      .collect();
    self.factory.computed_unknown(self.consumable(values))
  }
}

