use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::{Expression, YieldExpression};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_yield_expression(&mut self, node: &'a YieldExpression<'a>) -> H::Entity {
    self.refer_to_global();

    if let Some(argument) = &node.argument {
      let argument = self.exec_expression(argument);
      argument.consume(self);
    }
    self.factory.unknown()
  }
}
