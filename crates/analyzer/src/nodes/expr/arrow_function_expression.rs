use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::ArrowFunctionExpression;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_arrow_function_expression(
    &mut self,
    node: &'a ArrowFunctionExpression<'a>,
  ) -> H::Entity {
    self.host.new_arrow_function(node)
  }
}
