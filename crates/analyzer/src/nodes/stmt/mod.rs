pub mod block_statement;
pub mod break_statement;
pub mod continue_statement;
pub mod declaration;
pub mod do_while_statement;
pub mod for_in_statement;
pub mod for_of_statement;
pub mod for_statement;
pub mod if_statement;
pub mod labeled_statements;
pub mod module_declaration;
pub mod return_statement;
pub mod statement_vec;
pub mod switch_statement;
pub mod throw_statement;
pub mod try_statement;
pub mod while_statement;

use crate::{analyzer::Analyzer, host::Host};
use if_statement::TraverseIfStatement;
use oxc::ast::{ast::Statement, match_declaration, match_module_declaration};

#[allow(unused_variables)]
pub trait TraverseStatement<'a>: TraverseIfStatement<'a> {
  fn before_statement(&self, node: &'a Statement<'a>) {}
  fn after_statement(&self, node: &'a Statement<'a>) {}
}

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn declare_statement(&mut self, node: &'a Statement) {
    match node {
      match_declaration!(Statement) => {
        let node = node.to_declaration();
        self.declare_declaration(node, false);
      }
      match_module_declaration!(Statement) => {
        let node = node.to_module_declaration();
        self.declare_module_declaration(node);
      }
      Statement::LabeledStatement(node) => self.declare_labeled_statement(node),
      _ => {}
    }
  }

  pub fn init_statement(&mut self, node: &'a Statement) {
    self.host.before_statement(node);

    if !matches!(
      node,
      Statement::BlockStatement(_)
        | Statement::IfStatement(_)
        | Statement::WhileStatement(_)
        | Statement::DoWhileStatement(_)
        | Statement::ForStatement(_)
        | Statement::ForInStatement(_)
        | Statement::ForOfStatement(_)
        | Statement::SwitchStatement(_)
        | Statement::LabeledStatement(_)
        | Statement::TryStatement(_),
    ) {
      self.pending_labels.clear();
    }
    match node {
      match_declaration!(Statement) => {
        let node = node.to_declaration();
        self.init_declaration(node);
      }
      match_module_declaration!(Statement) => {
        let node = node.to_module_declaration();
        self.init_module_declaration(node);
      }
      Statement::ExpressionStatement(node) => {
        self.exec_expression(&node.expression);
      }
      Statement::BlockStatement(node) => self.exec_block_statement(node),
      Statement::IfStatement(node) => self.exec_if_statement(node),
      Statement::WhileStatement(node) => self.exec_while_statement(node),
      Statement::DoWhileStatement(node) => self.exec_do_while_statement(node),
      Statement::ForStatement(node) => self.exec_for_statement(node),
      Statement::ForInStatement(node) => self.exec_for_in_statement(node),
      Statement::ForOfStatement(node) => self.exec_for_of_statement(node),
      Statement::SwitchStatement(node) => self.exec_switch_statement(node),
      Statement::BreakStatement(node) => self.exec_break_statement(node),
      Statement::ContinueStatement(node) => self.exec_continue_statement(node),
      Statement::ReturnStatement(node) => self.exec_return_statement(node),
      Statement::LabeledStatement(node) => self.exec_labeled_statement(node),
      Statement::TryStatement(node) => self.exec_try_statement(node),
      Statement::ThrowStatement(node) => self.exec_throw_statement(node),
      Statement::EmptyStatement(_) => {}
      Statement::DebuggerStatement(_node) => {}
      Statement::WithStatement(_node) => unimplemented!("with statement"),
    }

    self.host.after_statement(node);
  }
}
