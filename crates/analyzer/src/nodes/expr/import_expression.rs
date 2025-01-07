use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::{Expression, ImportExpression};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_import_expression(&mut self, node: &'a ImportExpression<'a>) -> H::Entity {
    let mut deps = vec![];

    deps.push(self.exec_expression(&node.source).get_to_string(self));

    for argument in &node.arguments {
      deps.push(self.exec_expression(argument));
    }

    // FIXME: if have side effects, then consume all deps

    self.factory.computed_unknown(self.consumable(deps))
  }
}
