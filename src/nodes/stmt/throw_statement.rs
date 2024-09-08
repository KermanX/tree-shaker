use crate::{
  analyzer::Analyzer,
  entity::{dep::EntityDepNode, forwarded::ForwardedEntity},
  transformer::Transformer,
};
use oxc::{
  ast::ast::{Statement, ThrowStatement},
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub fn exec_throw_statement(&mut self, node: &'a ThrowStatement<'a>) {
    let value = self.exec_expression(&node.argument);

    let dep = self.new_entity_dep(EntityDepNode::ThrowStatement(node));

    let try_scope = self.try_scope_mut();
    try_scope.thrown_values.push(ForwardedEntity::new(value, dep));
    let cf_scope_index = try_scope.cf_scope_index;
    self.exit_to(cf_scope_index);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_throw_statement(&self, node: &'a ThrowStatement<'a>) -> Option<Statement<'a>> {
    let need_val = self.is_referred(EntityDepNode::ThrowStatement(&node));

    let ThrowStatement { span, argument, .. } = node;

    let argument_span = argument.span();

    let argument = self
      .transform_expression(argument, need_val)
      .unwrap_or_else(|| self.build_unused_expression(argument_span));

    Some(self.ast_builder.statement_throw(*span, argument))
  }
}