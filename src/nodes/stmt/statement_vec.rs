use crate::{analyzer::Analyzer, ast::AstType2, transformer::Transformer};
use oxc::{
  allocator::Vec,
  ast::ast::Statement,
  span::{GetSpan, Span},
};

const AST_TYPE: AstType2 = AstType2::BlockStatement;

#[derive(Debug, Default)]
pub struct StatementVecData {
  last_stmt: Option<Span>,
}

impl<'a> Analyzer<'a> {
  pub fn exec_statement_vec(
    &mut self,
    data: &mut StatementVecData,
    exited: Option<bool>,
    statements: &'a Vec<'a, Statement<'a>>,
  ) {
    let cf_scope_id = self.push_cf_scope(exited, false);
    self.push_variable_scope(cf_scope_id);

    let mut span: Option<Span> = None;
    for statement in statements {
      if self.cf_scope().must_exited() {
        break;
      }
      self.exec_statement(statement);
      span = Some(statement.span());
    }
    if let Some(span) = span {
      data.last_stmt = match data.last_stmt {
        Some(current_span) => Some(current_span.max(span)),
        None => Some(span),
      };
    }

    self.pop_variable_scope();
    self.pop_cf_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_statement_vec(
    &mut self,
    data: &StatementVecData,
    statements: Vec<'a, Statement<'a>>,
  ) -> Vec<'a, Statement<'a>> {
    let mut result = self.ast_builder.vec();

    if data.last_stmt.is_none() {
      return result;
    }

    for statement in statements {
      let span = statement.span();

      if let Some(statement) = self.transform_statement(statement) {
        result.push(statement);
      }

      if data.last_stmt == Some(span) {
        break;
      }
    }

    result
  }
}
