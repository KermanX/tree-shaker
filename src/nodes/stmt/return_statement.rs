use crate::{
  analyzer::Analyzer,
  entity::{dep::EntityDep, forwarded::ForwardedEntity, literal::LiteralEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{ReturnStatement, Statement};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_return_statement(&mut self, node: &'a ReturnStatement) {
    let indeterminate = self.indeterminate;
    let value = node
      .argument
      .as_ref()
      .map_or_else(|| LiteralEntity::new_undefined(), |expr| self.exec_expression(expr));
    self.function_scope_mut().on_return(
      indeterminate,
      ForwardedEntity::new(value, vec![EntityDep::ReturnStatement(node)]),
    );
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_return_statement(
    &self,
    node: ReturnStatement<'a>,
  ) -> Option<Statement<'a>> {
    let need_val = self.is_referred(EntityDep::ReturnStatement(&node));

    let ReturnStatement { span, argument } = node;

    Some(
      self
        .ast_builder
        .statement_return(span, argument.and_then(|arg| self.transform_expression(arg, need_val))),
    )
  }
}
