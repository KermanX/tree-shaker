use crate::{analyzer::Analyzer, ast::AstType2, transformer::Transformer};
use oxc::{
  ast::ast::{Statement, WhileStatement},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::WhileStatement;

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_loop: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_while_statement(&mut self, node: &'a WhileStatement<'a>) {
    // This may be indeterminate. However, we can't know it until we execute the test.
    // And there should be no same level break/continue statement in test.
    // `a: while(() => { break a }) { }` is illegal.
    let test = self.exec_expression(&node.test);

    let (need_loop, indeterminate) = match test.test_truthy() {
      Some(true) => (true, false),
      Some(false) => (false, false),
      None => (true, true),
    };

    let cf_scope_id = self.push_cf_scope(if indeterminate { None } else { Some(false) }, true);
    self.push_variable_scope(cf_scope_id);

    if need_loop {
      self.exec_statement(&node.body);
    }

    self.pop_variable_scope();
    self.pop_cf_scope();

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.need_loop |= need_loop;
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_while_statement(
    &mut self,
    node: WhileStatement<'a>,
  ) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let WhileStatement { span, test, body, .. } = node;
    let body_span = body.span();

    let test = self.transform_expression(test, data.need_loop);
    let body = data.need_loop.then(|| self.transform_statement(body)).flatten();

    match (test, body) {
      (Some(test), body) => Some(self.ast_builder.statement_while(
        span,
        test,
        body.unwrap_or_else(|| self.ast_builder.statement_empty(body_span)),
      )),
      (None, Some(_)) => unreachable!(),
      (None, None) => None,
    }
  }
}
