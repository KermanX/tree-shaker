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
  pub fn exec_if_statement(&mut self, node: &'a IfStatement) {
    let test = self.exec_expression(&node.test);

    let (maybe_true, maybe_false) = match test.test_truthy() {
      Some(true) => (true, false),
      Some(false) => (false, true),
      None => (true, true),
    };

    let indeterminate = maybe_true && maybe_false;

    if indeterminate {
      test.consume_self(self);
      self.push_cf_scope(None, false);
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
  pub fn transform_if_statement(&self, node: IfStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let IfStatement { span, test, consequent, alternate, .. } = node;

    let consequent = self.transform_statement(consequent);
    let alternate = alternate.and_then(|alt| self.transform_statement(alt));

    let need_test_val =
      data.maybe_true && data.maybe_false && (consequent.is_some() || alternate.is_some());
    let test = self.transform_expression(test, need_test_val);

    let mut statements = self.ast_builder.vec();

    match (data.maybe_true, data.maybe_false) {
      (true, true) => {
        // Both cases are possible
        return match (consequent, alternate) {
          (Some(consequent), alternate) => {
            Some(self.ast_builder.statement_if(span, test.unwrap(), consequent, alternate))
          }
          (None, Some(alternate)) => Some(self.ast_builder.statement_if(
            span,
            self.build_negate_expression(test.unwrap()),
            alternate,
            None,
          )),
          (None, None) => test.map(|test| self.ast_builder.statement_expression(test.span(), test)),
        };
      }
      (true, false) => {
        // Only one case is possible
        test.map(|test| statements.push(self.ast_builder.statement_expression(test.span(), test)));
        consequent.map(|body| statements.push(body));
      }
      (false, true) => {
        // Only one case is possible
        test.map(|test| statements.push(self.ast_builder.statement_expression(test.span(), test)));
        alternate.map(|body| statements.push(body));
      }
      (false, false) => unreachable!(),
    };

    if statements.is_empty() {
      None
    } else {
      Some(self.ast_builder.statement_block(span, statements))
    }
  }
}
