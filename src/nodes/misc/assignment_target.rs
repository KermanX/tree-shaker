use crate::{analyzer::Analyzer, ast::AstType2, entity::entity::Entity, transformer::Transformer};
use oxc::{
  ast::{ast::AssignmentTarget, match_assignment_target_pattern, match_simple_assignment_target},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::AssignmentTarget;

#[derive(Debug, Default)]
pub struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_assignment_target(
    &mut self,
    node: &'a AssignmentTarget<'a>,
    (effect, value): (bool, Entity<'a>),
  ) {
    if effect {
      let data = self.load_data::<Data>(AstType2::BindingPattern, node);
      data.has_effect = true;
    }
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
  pub fn transform_assignment_target(
    &self,
    node: &'a AssignmentTarget<'a>,
  ) -> Option<AssignmentTarget<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let transformed = match node {
      match_simple_assignment_target!(AssignmentTarget) => self
        .transform_simple_assignment_target_write(node.to_simple_assignment_target())
        .map(|node| self.ast_builder.assignment_target_simple(node)),
      match_assignment_target_pattern!(AssignmentTarget) => self
        .transform_assignment_target_pattern(node.to_assignment_target_pattern())
        .map(|node| self.ast_builder.assignment_target_assignment_target_pattern(node)),
    };

    if data.has_effect {
      Some(transformed.unwrap_or_else(|| self.build_unused_assignment_target(node.span())))
    } else {
      transformed
    }
  }
}
