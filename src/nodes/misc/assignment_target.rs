use crate::{analyzer::Analyzer, ast::AstType2, entity::entity::Entity, transformer::Transformer};
use oxc::{
  ast::{ast::AssignmentTarget, match_assignment_target_pattern, match_simple_assignment_target},
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub fn exec_assignment_target(&mut self, node: &'a AssignmentTarget<'a>, value: Entity<'a>) {
    match node {
      match_simple_assignment_target!(AssignmentTarget) => {
        self.exec_simple_assignment_target_write(node.to_simple_assignment_target(), value);
      }
      match_assignment_target_pattern!(AssignmentTarget) => {
        self.exec_assignment_target_pattern(node.to_assignment_target_pattern(), value);
      }
    }
  }
}

impl<'a> Transformer<'a> {
  // (is_empty, node)
  pub fn transform_assignment_target(
    &self,
    node: &'a AssignmentTarget<'a>,
    in_rest: bool,
  ) -> (bool, Option<AssignmentTarget<'a>>) {
    let transformed = match node {
      match_simple_assignment_target!(AssignmentTarget) => self
        .transform_simple_assignment_target_write(node.to_simple_assignment_target())
        .map(|node| self.ast_builder.assignment_target_simple(node)),
      match_assignment_target_pattern!(AssignmentTarget) => self
        .transform_assignment_target_pattern(node.to_assignment_target_pattern())
        .map(|node| self.ast_builder.assignment_target_assignment_target_pattern(node)),
    };

    if data.has_effect && transformed.is_none() {
      let span = node.span();
      let unused = if in_rest {
        self.build_unused_assignment_target_in_rest(span)
      } else {
        self.build_unused_assignment_target(span)
      };
      (true, Some(unused))
    } else {
      (transformed.is_none(), transformed)
    }
  }
}
