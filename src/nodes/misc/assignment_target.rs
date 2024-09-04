use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::ast::{
  ast::AssignmentTarget, match_assignment_target_pattern, match_simple_assignment_target,
};

impl<'a> Analyzer<'a> {
  pub fn exec_assignment_target(&mut self, node: &'a AssignmentTarget<'a>, value: Entity<'a>) {
    match node {
      match_simple_assignment_target!(AssignmentTarget) => {
        self.exec_simple_assignment_target(node.to_simple_assignment_target(), value);
      }
      match_assignment_target_pattern!(AssignmentTarget) => {
        self.exec_assignment_target_pattern(node.to_assignment_target_pattern(), value);
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_assignment_target(
    &self,
    node: AssignmentTarget<'a>,
  ) -> Option<AssignmentTarget<'a>> {
    match node {
      match_simple_assignment_target!(AssignmentTarget) => {
        self.transform_simple_assignment_target(node.try_into().unwrap())
      }
      match_assignment_target_pattern!(AssignmentTarget) => {
        self.transform_assignment_target_pattern(node.try_into().unwrap())
      }
    }
  }
}
