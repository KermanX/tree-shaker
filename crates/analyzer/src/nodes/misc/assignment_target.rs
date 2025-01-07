use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  ast::{
    ast::{AssignmentTarget, Expression},
    match_assignment_target_pattern, match_simple_assignment_target,
  },
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_assignment_target_read(
    &mut self,
    node: &'a AssignmentTarget<'a>,
  ) -> (H::Entity, Option<(H::Entity, H::Entity)>) {
    match node {
      match_simple_assignment_target!(AssignmentTarget) => {
        self.exec_simple_assignment_target_read(node.to_simple_assignment_target())
      }
      match_assignment_target_pattern!(AssignmentTarget) => {
        unreachable!()
      }
    }
  }

  pub fn exec_assignment_target_write(
    &mut self,
    node: &'a AssignmentTarget<'a>,
    value: H::Entity,
    cache: Option<(H::Entity, H::Entity)>,
  ) {
    match node {
      match_simple_assignment_target!(AssignmentTarget) => {
        self.exec_simple_assignment_target_write(node.to_simple_assignment_target(), value, cache);
      }
      match_assignment_target_pattern!(AssignmentTarget) => {
        self.exec_assignment_target_pattern_write(node.to_assignment_target_pattern(), value);
      }
    }
  }
}
