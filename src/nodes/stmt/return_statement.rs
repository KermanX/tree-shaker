use crate::{analyzer::Analyzer, entity::LiteralEntity, transformer::Transformer};
use oxc::ast::{
  ast::{ReturnStatement, Statement},
  AstKind,
};

impl<'a> Analyzer<'a> {
  pub fn exec_return_statement(&mut self, node: &'a ReturnStatement) {
    let value = node
      .argument
      .as_ref()
      .map_or_else(|| LiteralEntity::new_undefined(), |expr| self.exec_expression(expr));
    let dep = AstKind::ReturnStatement(node);
    self.return_value(value, dep);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_return_statement(&self, node: &'a ReturnStatement<'a>) -> Option<Statement<'a>> {
    let need_val = self.is_referred(AstKind::ReturnStatement(&node));

    let ReturnStatement { span, argument } = node;

    Some(self.ast_builder.statement_return(
      *span,
      argument.as_ref().and_then(|arg| self.transform_expression(arg, need_val)),
    ))
  }
}
