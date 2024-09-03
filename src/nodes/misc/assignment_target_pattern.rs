use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::ast::ast::{AssignmentTarget, AssignmentTargetPattern};

impl<'a> Analyzer<'a> {
  pub fn exec_assignment_target_pattern(
    &mut self,
    node: &'a AssignmentTargetPattern<'a>,
    value: Entity<'a>,
  ) -> Entity<'a> {
    todo!()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_assignment_target_pattern(
    &mut self,
    node: AssignmentTargetPattern<'a>,
  ) -> Option<AssignmentTarget<'a>> {
    todo!()
  }
}
