use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::{ArrayExpression, ArrayExpressionElement};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_array_expression(&mut self, node: &'a ArrayExpression<'a>) -> H::ArrayEntity {
    let mut array = self.host.new_empty_array(node);

    for node in &node.elements {
      match node {
        ArrayExpressionElement::SpreadElement(node) => {
          self.host.init_spread(node, &mut array, self.exec_spread_element(node));
        }
        ArrayExpressionElement::Elision(_node) => {
          self.host.init_element(node, &mut array, self.host.new_undefined());
        }
        _ => {
          self.host.init_element(node, &mut array, self.exec_expression(node.to_expression()));
        }
      }
    }

    array
  }
}
