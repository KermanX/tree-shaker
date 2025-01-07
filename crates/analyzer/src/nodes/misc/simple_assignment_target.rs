use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::{
  ast::{Expression, SimpleAssignmentTarget},
  match_member_expression,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_simple_assignment_target_read(
    &mut self,
    node: &'a SimpleAssignmentTarget<'a>,
  ) -> (H::Entity, Option<(H::Entity, H::Entity)>) {
    match node {
      match_member_expression!(SimpleAssignmentTarget) => {
        let (value, cache) = self.exec_member_expression_read(node.to_member_expression(), true);
        (value, Some(cache))
      }
      SimpleAssignmentTarget::AssignmentTargetIdentifier(node) => {
        (self.exec_identifier_reference_read(node), None)
      }
      _ => unreachable!(),
    }
  }

  pub fn exec_simple_assignment_target_write(
    &mut self,
    node: &'a SimpleAssignmentTarget<'a>,
    value: H::Entity,
    cache: Option<(H::Entity, H::Entity)>,
  ) {
    match node {
      match_member_expression!(SimpleAssignmentTarget) => {
        self.exec_member_expression_write(node.to_member_expression(), value, cache)
      }
      SimpleAssignmentTarget::AssignmentTargetIdentifier(node) => {
        self.exec_identifier_reference_write(node, value)
      }
      _ => unreachable!(),
    }
  }
}

