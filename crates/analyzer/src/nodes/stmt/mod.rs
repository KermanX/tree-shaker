mod return_statement;

use crate::EcmaAnalyzer;
use oxc::ast::ast::Statement;
pub use return_statement::*;

pub trait StatementAnalyzer<'a>: ReturnStatementAnalyzer<'a> {
  fn before_declare_statement(&mut self, node: &'a Statement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
  }

  fn after_declare_statement(&mut self, node: &'a Statement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
  }

  fn declare_statement(&mut self, node: &'a Statement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.before_declare_statement(node);
    // match node {
    //   match_declaration!(Statement) => {
    //     let node = node.to_declaration();
    //     self.declare_declaration(node, false);
    //   }
    //   match_module_declaration!(Statement) => {
    //     let node = node.to_module_declaration();
    //     self.declare_module_declaration(node);
    //   }
    //   Statement::LabeledStatement(node) => self.declare_labeled_statement(node),
    //   _ => {}
    // }
    self.after_declare_statement(node);
  }

  fn before_init_statement(&mut self, node: &'a Statement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
  }

  fn after_init_statement(&mut self, node: &'a Statement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
  }

  fn init_statement(&mut self, node: &'a Statement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.before_init_statement(node);
    match node {
      Statement::ExpressionStatement(node) => {
        self.exec_expression(&node.expression);
      }
      Statement::ReturnStatement(node) => self.exec_return_statement(node),
      _ => todo!(),
    }
    self.after_init_statement(node);
  }
}
