use crate::{
  analyzer::Analyzer,
  entity::{dep::EntityDepNode, entity::Entity, forwarded::ForwardedEntity},
  transformer::Transformer,
};
use oxc::ast::{
  ast::{Expression, SimpleAssignmentTarget},
  match_member_expression,
};

impl<'a> Analyzer<'a> {
  pub fn exec_simple_assignment_target_read(
    &mut self,
    node: &'a SimpleAssignmentTarget<'a>,
  ) -> Entity<'a> {
    match node {
      match_member_expression!(SimpleAssignmentTarget) => {
        self.exec_member_expression_read(node.to_member_expression())
      }
      SimpleAssignmentTarget::AssignmentTargetIdentifier(node) => {
        self.exec_identifier_reference_read(node)
      }
      _ => unreachable!(),
    }
  }

  pub fn exec_simple_assignment_target_write(
    &mut self,
    node: &'a SimpleAssignmentTarget<'a>,
    value: Entity<'a>,
  ) {
    match node {
      match_member_expression!(SimpleAssignmentTarget) => {
        self.exec_member_expression_write(node.to_member_expression(), value)
      }
      SimpleAssignmentTarget::AssignmentTargetIdentifier(node) => {
        self.exec_identifier_reference_write(node, value)
      }
      _ => unreachable!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_simple_assignment_target_read(
    &self,
    node: &'a SimpleAssignmentTarget<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    match node {
      match_member_expression!(SimpleAssignmentTarget) => {
        self.transform_member_expression_read(node.to_member_expression(), need_val)
      }
      SimpleAssignmentTarget::AssignmentTargetIdentifier(node) => {
        let inner = self.transform_identifier_reference_read(node, need_val);
        inner.map(|inner| self.ast_builder.expression_from_identifier_reference(inner))
      }
      _ => unreachable!(),
    }
  }

  pub fn transform_simple_assignment_target_write(
    &self,
    node: &'a SimpleAssignmentTarget<'a>,
  ) -> Option<SimpleAssignmentTarget<'a>> {
    match node {
      match_member_expression!(SimpleAssignmentTarget) => self
        .transform_member_expression_write(node.to_member_expression())
        .map(|node| self.ast_builder.simple_assignment_target_member_expression(node)),
      SimpleAssignmentTarget::AssignmentTargetIdentifier(node) => {
        let inner = self.transform_identifier_reference_write(node);
        inner
          .map(|inner| self.ast_builder.simple_assignment_target_from_identifier_reference(inner))
      }
      _ => unreachable!(),
    }
  }
}
