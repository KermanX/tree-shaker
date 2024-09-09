use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::ast::ast::AssignmentTargetRest;

impl<'a> Analyzer<'a> {
  pub fn exec_assignment_target_rest(
    &mut self,
    node: &'a AssignmentTargetRest<'a>,
    value: Entity<'a>,
  ) {
    self.exec_assignment_target(&node.target, (true, value))
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_assignment_target_rest(
    &self,
    node: &'a AssignmentTargetRest<'a>,
  ) -> Option<AssignmentTargetRest<'a>> {
    let AssignmentTargetRest { span, target } = node;

    let target = self.transform_assignment_target(target).unwrap();

    Some(self.ast_builder.assignment_target_rest(*span, target))
  }
}
