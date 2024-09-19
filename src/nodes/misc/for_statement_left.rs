use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::ast::ast::ForStatementLeft;

impl<'a> Analyzer<'a> {
  pub fn exec_for_statement_left(&mut self, node: &'a ForStatementLeft<'a>, init: Entity<'a>) {
    match node {
      ForStatementLeft::VariableDeclaration(node) => {
        self.declare_variable_declaration(node, false);
        self.init_variable_declaration(node, Some(init));
      }
      ForStatementLeft::UsingDeclaration(_node) => unreachable!(),
      _ => self.exec_assignment_target_write(node.to_assignment_target(), init, None),
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_for_statement_left(
    &self,
    node: &'a ForStatementLeft<'a>,
  ) -> Option<ForStatementLeft<'a>> {
    match node {
      ForStatementLeft::VariableDeclaration(node) => self
        .transform_variable_declaration(node)
        .map(|decl| self.ast_builder.for_statement_left_from_variable_declaration(decl)),
      ForStatementLeft::UsingDeclaration(_node) => unreachable!(),
      _ => self
        .transform_assignment_target_write(node.to_assignment_target(), false, false)
        .1
        .map(|target| self.ast_builder.for_statement_left_assignment_target(target)),
    }
  }
}
