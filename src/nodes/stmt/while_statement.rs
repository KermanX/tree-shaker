use crate::{analyzer::Analyzer, ast::AstKind2, scope::CfScopeKind, transformer::Transformer};
use oxc::{
  ast::ast::{Statement, WhileStatement},
  span::GetSpan,
};

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_loop: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_while_statement(&mut self, node: &'a WhileStatement<'a>) {
    let labels = self.take_labels();

    // This may be indeterminate. However, we can't know it until we execute the test.
    // And there should be no same level break/continue statement in test.
    // `a: while(() => { break a }) { }` is illegal.
    let test = self.exec_expression(&node.test);

    if test.test_truthy() == Some(false) {
      return;
    }
    test.consume(self);

    let data = self.load_data::<Data>(AstKind2::WhileStatement(node));
    data.need_loop = true;

    self.push_cf_scope(CfScopeKind::BreakableWithoutLabel, labels.clone(), Some(false));
    self.exec_loop(move |analyzer| {
      analyzer.push_cf_scope(CfScopeKind::Continuable, labels.clone(), None);

      analyzer.exec_statement(&node.body);
      analyzer.exec_expression(&node.test).consume(analyzer);

      analyzer.pop_cf_scope();
    });
    self.pop_cf_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_while_statement(&self, node: &'a WhileStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AstKind2::WhileStatement(node));

    let WhileStatement { span, test, body, .. } = node;
    let body_span = body.span();

    let test = self.transform_expression(test, data.need_loop);
    let body = data.need_loop.then(|| self.transform_statement(body)).flatten();

    match (test, body) {
      (Some(test), body) => Some(self.ast_builder.statement_while(
        *span,
        test,
        body.unwrap_or_else(|| self.ast_builder.statement_empty(body_span)),
      )),
      (None, Some(_)) => unreachable!(),
      (None, None) => None,
    }
  }
}
