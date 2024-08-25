use crate::entity::{entity::Entity, literal::LiteralEntity, union::UnionEntity};
use oxc::semantic::ScopeId;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub(crate) struct FunctionScope<'a> {
  pub(crate) id: ScopeId,
  pub(crate) has_effect: bool,
  /// `None` for indeterminate
  pub(crate) returned: Option<bool>,
  pub(crate) returned_value: Vec<Entity<'a>>,
}

static FUNCTION_SCOPE_ID: AtomicU32 = AtomicU32::new(0);

impl<'a> FunctionScope<'a> {
  pub(crate) fn new() -> Self {
    FunctionScope {
      id: ScopeId::new(FUNCTION_SCOPE_ID.fetch_add(1, Ordering::Relaxed)),
      has_effect: false,
      returned: Some(false),
      returned_value: Vec::new(),
    }
  }

  pub(crate) fn get_result(self) -> (bool, Entity<'a>) {
    (
      self.has_effect,
      if self.returned_value.is_empty() {
        LiteralEntity::new_undefined()
      } else {
        UnionEntity::new(self.returned_value)
      },
    )
  }

  pub(crate) fn on_return(&mut self, indeterminate: bool, value: Entity<'a>) {
    match self.returned {
      Some(true) => unreachable!(),
      Some(false) => {
        self.returned = indeterminate.then(|| true);
      }
      None => {}
    }
    self.returned_value.push(value);
  }
}
