use crate::{analyzer::Analyzer, entity::dep::EntityDepNode, transformer::Transformer};
use oxc::ast::ast::{LabeledStatement, Statement};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_labeled_statement(&mut self, node: &'a LabeledStatement<'a>) {
    self.current_label = Some(&node.label.name);
    self.exec_statement(&node.body);
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_labeled_statement(
    &mut self,
    node: LabeledStatement<'a>,
  ) -> Option<Statement<'a>> {
    let LabeledStatement { span, label, body } = node;

    let body = self.transform_statement(body);

    if self.is_referred(EntityDepNode::LabelIdentifier(&label)) {
      Some(self.ast_builder.statement_labeled(span, label, body.unwrap()))
    } else {
      body
    }
  }
}
