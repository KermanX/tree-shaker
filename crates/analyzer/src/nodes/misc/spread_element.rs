use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::ast::{ArrayExpressionElement, SpreadElement};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_spread_element(&mut self, node: &'a SpreadElement<'a>) -> H::Entity {
    let argument = self.exec_expression(&node.argument);
    argument.iterate_result_union(self, self.consumable(AstKind2::SpreadElement(node)))
  }
}

