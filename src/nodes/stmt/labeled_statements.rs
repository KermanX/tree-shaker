use crate::{analyzer::Analyzer, entity::label::LabelEntity, transformer::Transformer};
use oxc::ast::{
  ast::{LabeledStatement, Statement},
  AstKind,
};

impl<'a> Analyzer<'a> {
  pub fn declare_labeled_statement(&mut self, node: &'a LabeledStatement<'a>) {
    self.declare_statement(&node.body);
  }

  pub fn exec_labeled_statement(&mut self, node: &'a LabeledStatement<'a>) {
    self.pending_labels.push(LabelEntity::new(&node.label));
    self.exec_statement(&node.body);
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_labeled_statement(
    &self,
    node: &'a LabeledStatement<'a>,
  ) -> Option<Statement<'a>> {
    let LabeledStatement { span, label, body } = node;

    let body = self.transform_statement(body);

    if self.is_referred(AstKind::LabelIdentifier(&label)) {
      Some(self.ast_builder.statement_labeled(*span, label.clone(), body.unwrap()))
    } else {
      body
    }
  }
}
