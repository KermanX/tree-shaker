use crate::analyzer::Analyzer;
use ecma_analyzer::{EcmaAnalyzer, StatementAnalyzer};
use oxc::ast::ast::Statement;
mod return_statement;

impl<'a> StatementAnalyzer<'a> for Analyzer<'a> {
  fn before_declare_statement(&mut self, node: &'a Statement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.push_span(node);
  }

  fn after_declare_statement(&mut self, node: &'a Statement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.pop_span();
  }

  fn before_init_statement(&mut self, node: &'a Statement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.push_span(node);
  }

  fn after_init_statement(&mut self, node: &'a Statement<'a>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.pop_span();
  }
}
