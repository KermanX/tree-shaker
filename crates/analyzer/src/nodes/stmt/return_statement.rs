use crate::EcmaAnalyzer;
use oxc::ast::ast::ReturnStatement;

pub trait ReturnStatementAnalyzer<'a> {
  fn on_return_value(&mut self, node: &'a ReturnStatement<'a>, value: Self::Entity) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>;

  fn exec_return_statement(&mut self, node: &'a ReturnStatement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let value =
      node.argument.as_ref().map_or(self.new_undefined(), |expr| self.exec_expression(expr));

    let value = self.on_return_value(node, value);
    let call_scope = self.call_scope_mut();
    call_scope.returned_values.push(value);

    let target_depth = call_scope.cf_scope_depth;
    self.exit_to(target_depth);
  }
}
