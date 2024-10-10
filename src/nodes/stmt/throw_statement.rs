use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::{
  ast::{
    ast::{Statement, ThrowStatement},
    AstKind,
  },
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub fn exec_throw_statement(&mut self, node: &'a ThrowStatement<'a>) {
    let value = self.exec_expression(&node.argument);

    let dep = box_consumable(AstKind::ThrowStatement(node));

    self.explicit_throw(self.factory.new_computed(value, dep));
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_throw_statement(&self, node: &'a ThrowStatement<'a>) -> Option<Statement<'a>> {
    let need_val = self.is_referred(AstKind::ThrowStatement(&node));

    let ThrowStatement { span, argument, .. } = node;

    let argument_span = argument.span();

    let argument = self
      .transform_expression(argument, need_val)
      .unwrap_or_else(|| self.build_unused_expression(argument_span));

    Some(self.ast_builder.statement_throw(*span, argument))
  }
}
