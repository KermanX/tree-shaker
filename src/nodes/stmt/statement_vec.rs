use crate::{analyzer::Analyzer, data::StatementVecData, transformer::Transformer};
use oxc::{
  allocator::Vec,
  ast::{ast::Statement, match_declaration, match_module_declaration},
};

impl<'a> Analyzer<'a> {
  pub fn exec_statement_vec(
    &mut self,
    data: &mut StatementVecData,
    statements: &'a Vec<'a, Statement<'a>>,
  ) {
    for statement in statements {
      self.declare_statement(statement);
    }

    let mut last_stmt = None;
    for (index, statement) in statements.iter().enumerate() {
      if self.cf_scope().borrow().must_exited() {
        break;
      }
      self.init_statement(statement);
      last_stmt = Some(index);
    }
    if let Some(last_stmt) = last_stmt {
      data.last_stmt = match data.last_stmt {
        Some(old_last_stmt) => Some(old_last_stmt.max(last_stmt)),
        None => Some(last_stmt),
      };
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_statement_vec(
    &self,
    data: &StatementVecData,
    statements: &'a Vec<'a, Statement<'a>>,
  ) -> Vec<'a, Statement<'a>> {
    let mut result = self.ast_builder.vec();

    if data.last_stmt.is_none() {
      return result;
    }

    let mut exited = false;
    for (index, statement) in statements.iter().enumerate() {
      if !exited || self.is_declaration(statement) {
        if let Some(statement) = self.transform_statement(statement) {
          result.push(statement);
        }
      }

      if data.last_stmt == Some(index) {
        exited = true;
      }
    }

    result
  }

  fn is_declaration(&self, statement: &'a Statement<'a>) -> bool {
    match statement {
      match_declaration!(Statement) => true,
      match_module_declaration!(Statement) => true,
      Statement::LabeledStatement(node) => self.is_declaration(&node.body),
      _ => false,
    }
  }
}
