use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  ast::ast::{AssignmentTargetMaybeDefault, AssignmentTargetWithDefault},
  span::GetSpan,
};

#[derive(Debug, Default)]
pub struct WithDefaultData {
  need_init: bool,
}

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_assignment_target_maybe_default(
    &mut self,
    node: &'a AssignmentTargetMaybeDefault<'a>,
    value: H::Entity,
  ) {
    match node {
      AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(node) => {
        let (need_init, value) = self.exec_with_default(&node.init, value);

        data.need_init |= need_init;

        self.exec_assignment_target_write(&node.binding, value, None);
      }
      _ => self.exec_assignment_target_write(node.to_assignment_target(), value, None),
    }
  }
}
