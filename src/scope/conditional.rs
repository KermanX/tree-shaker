use super::CfScopeKind;
use crate::{
  analyzer::Analyzer,
  consumable::box_consumable,
  entity::{Entity, EntityDepNode},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct ConditionalData<'a> {
  pub determinate_tests: Rc<RefCell<Vec<Entity<'a>>>>,
  pub referred: bool,
}

impl<'a> Analyzer<'a> {
  pub fn push_conditional_cf_scope(
    &mut self,
    data: &mut ConditionalData<'a>,
    kind: CfScopeKind,
    test: Entity<'a>,
    historical_indeterminate: bool,
    current_indeterminate: bool,
  ) {
    let dep_node = EntityDepNode::from_data(data);
    let deps = if historical_indeterminate {
      if data.referred || self.is_referred(dep_node) {
        data.referred = true;
        self.consume(test);
        vec![]
      } else {
        data.determinate_tests.borrow_mut().push(test);
        vec![box_consumable((dep_node, data.determinate_tests.clone()))]
      }
    } else {
      data.determinate_tests.borrow_mut().push(test);
      vec![box_consumable(dep_node)]
    };
    self.push_cf_scope_with_dep(
      kind,
      None,
      deps,
      if current_indeterminate { None } else { Some(false) },
    );
  }
}
