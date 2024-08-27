use crate::{analyzer::Analyzer, ast::AstType2, transformer::Transformer};
use oxc::{
  ast::ast::{IfStatement, Statement},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::IfStatement;

#[derive(Debug, Default, Clone)]
pub struct Data {
  maybe_true: bool,
  maybe_false: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_if_statement(&mut self, node: &'a IfStatement) {
    let test = self.exec_expression(&node.test);

    let (maybe_true, maybe_false, indeterminate) = match test.test_truthy() {
      Some(true) => (true, false, false),
      Some(false) => (false, true, false),
      None => (true, true, true),
    };

    if indeterminate {
      self.push_cf_scope(None);
    }

    if maybe_true {
      self.exec_statement(&node.consequent);
    }
    if maybe_false {
      if let Some(alternate) = &node.alternate {
        self.exec_statement(alternate);
      }
    }

    if indeterminate {
      self.pop_cf_scope();
    }

    let data = self.load_data::<Data>(AST_TYPE, node);

    data.maybe_true |= maybe_true;
    data.maybe_false |= maybe_false;
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_if_statement(&mut self, node: IfStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let IfStatement { span, test, consequent, alternate, .. } = node;

    let test = self.transform_expression(test, data.maybe_true && data.maybe_false);
    let consequent = data.maybe_true.then(|| self.transform_statement(consequent));
    let alternate =
      data.maybe_false.then(|| alternate.and_then(|alt| self.transform_statement(alt)));

    let mut statements = self.ast_builder.vec();

    match (consequent, alternate) {
      (Some(consequent), Some(alternate)) => {
        // Both cases are possible
        let test = test.unwrap();
        match (consequent, alternate) {
          (Some(consequent), Some(alternate)) => {
            statements.push(self.ast_builder.statement_if(span, test, consequent, Some(alternate)));
          }
          (Some(consequent), None) => {
            statements.push(self.ast_builder.statement_if(span, test, consequent, None));
          }
          (None, Some(alternate)) => {
            statements.push(self.ast_builder.statement_if(
              span,
              self.build_negate_expression(test),
              alternate,
              None,
            ));
          }
          (None, None) => statements.push(self.ast_builder.statement_expression(test.span(), test)),
        }
      }
      (Some(body), None) | (None, Some(body)) => {
        // Only one case is possible
        test.map(|test| statements.push(self.ast_builder.statement_expression(test.span(), test)));
        body.map(|body| statements.push(body));
      }
      (None, None) => unreachable!(),
    };

    if statements.is_empty() {
      None
    } else {
      Some(self.ast_builder.statement_block(span, statements))
    }
  }
}
