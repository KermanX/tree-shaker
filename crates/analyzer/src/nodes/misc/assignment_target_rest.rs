use crate::{analyzer::Analyzer, host::Host};
use oxc::ast::ast::AssignmentTargetRest;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_assignment_target_rest(
    &mut self,
    node: &'a AssignmentTargetRest<'a>,
    value: H::Entity,
  ) {
    self.exec_assignment_target_write(&node.target, value, None)
  }
}
