use crate::{host::Host, analyzer::Analyzer,  dep::DepId};
use oxc::ast::ast::{ReturnStatement, Statement};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_return_statement(&mut self, node: &'a ReturnStatement) {
    let value =
      node.argument.as_ref().map_or(self.factory.undefined, |expr| self.exec_expression(expr));
    let dep = DepId::from(AstKind2::ReturnStatement(node));
    self.return_value(value, dep);
  }
}

