use super::{cf_scope::CfScopes, variable_scope::VariableScopes};
use crate::analyzer::Analyzer;
use std::rc::Rc;

pub fn find_first_different<T>(a: &Vec<Rc<T>>, b: &Vec<Rc<T>>) -> usize {
  for (index, this) in a.iter().enumerate() {
    if let Some(other) = b.get(index) {
      if !Rc::ptr_eq(this, other) {
        return index;
      }
    } else {
      return index;
    }
  }
  a.len()
}

impl<'a> Analyzer<'a> {
  pub fn find_first_different_cf_scope(&self, cf_scopes_2: &CfScopes<'a>) -> usize {
    find_first_different(&self.scope_context.cf_scopes, cf_scopes_2)
  }

  pub fn find_first_different_variable_scope(
    &self,
    variable_scopes_2: &VariableScopes<'a>,
  ) -> usize {
    find_first_different(&self.scope_context.variable_scopes, variable_scopes_2)
  }

  pub fn is_relatively_indeterminate(
    &self,
    first_different: usize,
    cf_scopes_2: &CfScopes<'a>,
  ) -> bool {
    self.scope_context.cf_scopes[first_different..]
      .iter()
      .chain(cf_scopes_2[first_different..].iter())
      .any(|s| s.borrow().is_indeterminate())
  }

  pub fn is_assignment_indeterminate(&self, cf_scopes_2: &CfScopes<'a>) -> bool {
    let first_different = self.find_first_different_cf_scope(cf_scopes_2);
    self.is_relatively_indeterminate(first_different, cf_scopes_2)
  }
}
