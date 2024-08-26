use super::loop_scope::LoopScope;
use crate::entity::{entity::Entity, literal::LiteralEntity, union::UnionEntity};

#[derive(Debug)]
pub(crate) struct FunctionScope<'a> {
  /// `None` for indeterminate
  pub returned: Option<bool>,
  pub returned_value: Vec<Entity<'a>>,
  pub loop_scopes: Vec<LoopScope<'a>>,
}

impl<'a> FunctionScope<'a> {
  pub(crate) fn new() -> Self {
    FunctionScope { returned: Some(false), returned_value: Vec::new(), loop_scopes: Vec::new() }
  }

  pub(crate) fn ret_val(self) -> Entity<'a> {
    if self.returned_value.is_empty() {
      LiteralEntity::new_undefined()
    } else {
      UnionEntity::new(self.returned_value)
    }
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
