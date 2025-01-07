use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::{Expression, ThisExpression};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_this_expression(&mut self, _node: &'a ThisExpression) -> H::Entity {
    self.get_this()
  }
}
