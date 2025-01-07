use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  ast::ast::{Expression, TemplateElementValue, TemplateLiteral},
  span::{GetSpan, SPAN},
};
use std::mem;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_template_literal(&mut self, node: &'a TemplateLiteral<'a>) -> H::Entity {
    let mut result = self.factory.string(node.quasi().unwrap().as_str());
    for (index, expression) in node.expressions.iter().enumerate() {
      let expression = self.exec_expression(expression);
      let quasi =
        self.factory.string(node.quasis.get(index + 1).unwrap().value.cooked.as_ref().unwrap());
      result = self.entity_op.add(self, result, expression);
      result = self.entity_op.add(self, result, quasi);
    }
    result
  }
}
