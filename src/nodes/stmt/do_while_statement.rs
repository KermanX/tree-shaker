use crate::{analyzer::Analyzer, ast::AstType2, transformer::Transformer};
use oxc::{
  ast::ast::{DoWhileStatement, Statement},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::DoWhileStatement;

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_test: bool,
  need_loop: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_do_while_statement(&mut self, node: &'a DoWhileStatement<'a>) {
    // Execute the first round.
    self.push_cf_scope_breakable(Some(false));
    self.exec_statement(&node.body);
    let cf_scope = self.pop_cf_scope();

    // FIXME: continue?
    if cf_scope.must_exited() {
      return;
    }

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.need_test = true;
    let test = self.exec_expression(&node.test);

    // The rest is the same as while statement.
    if test.test_truthy() == Some(false) {
      return;
    }
    test.consume_self(self);

    data.need_loop = true;

    self.exec_exhaustively(|analyzer| {
      analyzer.push_cf_scope_breakable(None);

      analyzer.exec_statement(&node.body);
      analyzer.exec_expression(&node.test).consume_self(analyzer);

      analyzer.pop_cf_scope();
    });
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_do_while_statement(
    &self,
    node: &'a DoWhileStatement<'a>,
  ) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let DoWhileStatement { span, test, body, .. } = node;
    let body_span = body.span();

    let body = self.transform_statement(body);

    if !data.need_test {
      body
    } else {
      let test = self.transform_expression(test, data.need_loop);
      if !data.need_loop {
        match (body, test) {
          (Some(body), Some(test)) => {
            let mut statements = self.ast_builder.vec();
            statements.push(body);
            statements.push(self.ast_builder.statement_expression(*span, test));
            Some(self.ast_builder.statement_block(*span, statements))
          }
          (None, Some(test)) => Some(self.ast_builder.statement_expression(*span, test)),
          (Some(body), None) => Some(body),
          (None, None) => None,
        }
      } else {
        Some(self.ast_builder.statement_do_while(
          *span,
          body.unwrap_or_else(|| self.ast_builder.statement_empty(body_span)),
          test.unwrap(),
        ))
      }
    }
  }
}
