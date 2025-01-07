use crate::{
  ast::{Arguments, AstKind2},
  entity::Entity,
  host::Host,
  Analyzer,
};
use oxc::{
  ast::ast::{Argument, Expression},
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_arguments(&mut self, node: &'a Arguments<'a>) -> H::Entity {
    let mut arguments = vec![];
    for argument in node {
      let (spread, val) = match argument {
        Argument::SpreadElement(node) => (true, self.exec_expression(&node.argument)),
        node => (false, self.exec_expression(node.to_expression())),
      };
      let dep = AstKind2::Argument(argument);
      arguments.push((spread, self.factory.computed(val, dep)));
    }
    self.factory.arguments(arguments)
  }
}
