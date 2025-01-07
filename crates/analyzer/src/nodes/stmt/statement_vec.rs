use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  allocator::Vec,
  ast::{ast::Statement, AstKind},
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn init_statement_vec(&mut self, node: AstKind<'a>, statements: &'a Vec<'a, Statement<'a>>) {
    for statement in statements {
      self.declare_statement(statement);
    }

    let mut last_stmt = None;
    for (index, statement) in statements.iter().enumerate() {
      if self.cf_scope().must_exited() {
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
