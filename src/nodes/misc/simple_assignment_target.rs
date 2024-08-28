use crate::{
  analyzer::Analyzer,
  entity::{dep::EntityDepNode, entity::Entity, forwarded::ForwardedEntity},
  transformer::Transformer,
};
use oxc::ast::{
  ast::{AssignmentTarget, SimpleAssignmentTarget},
  match_member_expression,
};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_simple_assignment_target(
    &mut self,
    node: &'a SimpleAssignmentTarget<'a>,
    value: Entity<'a>,
  ) {
    let dep = self.new_entity_dep(EntityDepNode::SimpleAssignmentTarget(node));
    match node {
      match_member_expression!(SimpleAssignmentTarget) => {
        todo!()
      }
      SimpleAssignmentTarget::AssignmentTargetIdentifier(node) => {
        self.exec_identifier_reference_write(node, ForwardedEntity::new(value, dep))
      }
      _ => unreachable!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_simple_assignment_target(
    &mut self,
    node: SimpleAssignmentTarget<'a>,
  ) -> Option<AssignmentTarget<'a>> {
    let referred = self.is_referred(EntityDepNode::SimpleAssignmentTarget(&node));
    match node {
      match_member_expression!(SimpleAssignmentTarget) => {
        todo!()
      }
      SimpleAssignmentTarget::AssignmentTargetIdentifier(node) => {
        let inner = self.transform_identifier_reference_write(node.unbox(), referred);
        inner.map(|inner| {
          self.ast_builder.assignment_target_simple(
            self.ast_builder.simple_assignment_target_from_identifier_reference(inner),
          )
        })
      }
      _ => unreachable!(),
    }
  }
}
