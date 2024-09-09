use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::{ast::ast::AssignmentTargetRest, span::GetSpan};

impl<'a> Analyzer<'a> {
  pub fn exec_assignment_target_rest(
    &mut self,
    node: &'a AssignmentTargetRest<'a>,
    effect_and_value: (bool, Entity<'a>),
  ) {
    self.exec_assignment_target(&node.target, effect_and_value)
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_assignment_target_rest(
    &self,
    node: &'a AssignmentTargetRest<'a>,
  ) -> Option<AssignmentTargetRest<'a>> {
    let AssignmentTargetRest { span, target } = node;

    let target_span = target.span();
    let target = self
      .transform_assignment_target(target)
      .unwrap_or_else(|| self.build_unused_assignment_target_for_rest(target_span));

    Some(self.ast_builder.assignment_target_rest(*span, target))
  }
}
