use crate::{analyzer::Analyzer, ast_type::AstType2, transformer::Transformer};
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
  pub(crate) fn exec_while_statement(&mut self, node: &'a WhileStatement) -> bool {
    let (test_effect, test_val) = self.exec_expression(&node.test);

    let (need_body, indeterminate) = match test_val.to_boolean() {
      Some(true) => (true, None),
      Some(false) => (false, None),
      None => (true, Some(self.start_indeterminate())),
    };

    let body_effect = need_body && self.exec_statement(&node.body);

    indeterminate.map(|prev| self.end_indeterminate(prev));

    let data = self.load_data::<Data>(AST_TYPE, node);

    data.need_loop |= test_effect || body_effect;

    test_effect || body_effect
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_while_statement(
    &self,
    node: WhileStatement<'a>,
  ) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let WhileStatement { span, test, body, .. } = node;
    let body_span = body.span();

    print!("data: {:?}\n", data);

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
