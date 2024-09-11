use super::{
  entity::Entity,
  literal::LiteralEntity,
  union::UnionEntity,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use crate::analyzer::Analyzer;
use oxc::semantic::ScopeId;

pub fn collect_effect_and_value<'a>(values: Vec<(bool, Entity<'a>)>) -> (bool, Entity<'a>) {
  let mut has_effect = false;
  let mut result = Vec::new();
  for (effect, value) in values {
    has_effect |= effect;
    result.push(value);
  }
  (has_effect, UnionEntity::new(result))
}

pub fn boolean_from_test_result<'a>(
  result: Option<bool>,
  deps: impl FnOnce() -> Vec<Entity<'a>>,
) -> Entity<'a> {
  match result {
    Some(value) => LiteralEntity::new_boolean(value),
    None => UnknownEntity::new_with_deps(UnknownEntityKind::Boolean, deps()),
  }
}

pub fn is_assignment_indeterminate<'a>(scope_path: &Vec<ScopeId>, analyzer: &Analyzer<'a>) -> bool {
  let mut var_scope_index = 0;
  for (index, scope) in analyzer.scope_context.variable_scopes.iter().enumerate() {
    let scope_id = scope.id;
    if scope_path.get(index).is_some_and(|id| *id == scope_id) {
      var_scope_index = index;
    } else {
      break;
    }
  }
  let target = analyzer.scope_context.variable_scopes[var_scope_index].cf_scope_index;
  analyzer.is_relatively_indeterminate(target)
}

#[macro_export]
macro_rules! use_consumed_flag {
  ($self: expr) => {
    if $self.consumed.get() {
      return;
    }
    $self.consumed.set(true);
  };
}
