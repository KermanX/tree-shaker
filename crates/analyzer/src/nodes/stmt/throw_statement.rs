use crate::{host::Host, analyzer::Analyzer};
use oxc::{
  ast::ast::{Statement, ThrowStatement},
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_throw_statement(&mut self, node: &'a ThrowStatement<'a>) {
    let value = self.exec_expression(&node.argument);

    let dep = self.consumable(AstKind2::ThrowStatement(node));

    self.explicit_throw(self.factory.computed(value, dep));
  }
}

