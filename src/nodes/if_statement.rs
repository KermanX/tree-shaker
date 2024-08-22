use crate::{analyzer::Analyzer, ast_type::AstType2, transformer::Transformer};
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
  pub(crate) fn exec_if_statement(&mut self, node: &'a IfStatement) -> bool {
    let (test_effect, test_val) = self.exec_expression(&node.test);

    let (maybe_true, maybe_false, indeterminate) = match test_val.to_boolean() {
      Some(true) => (true, false, None),
      Some(false) => (false, true, None),
      None => (true, true, Some(self.start_indeterminate())),
    };

    let mut effect = test_effect;
    if maybe_true {
      effect |= self.exec_statement(&node.consequent);
    }
    if maybe_false {
      if let Some(alternate) = &node.alternate {
        effect |= self.exec_statement(alternate);
      }
    }

    indeterminate.map(|prev| self.end_indeterminate(prev));

    self.set_data(AST_TYPE, node, Data { maybe_true, maybe_false });

    effect
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_if_statement(&self, node: IfStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let IfStatement { span, test, consequent, alternate, .. } = node;

    let consequent = data.maybe_true.then(|| self.transform_statement(consequent));
    let alternate =
      data.maybe_false.then(|| alternate.and_then(|alt| self.transform_statement(alt)));
    let test = self.transform_expression(test, data.maybe_true && data.maybe_false);

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
              self.negate_expression(test),
              alternate,
              None,
            ));
          }
          (None, None) => {}
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
