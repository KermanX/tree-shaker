use super::{cf_scope::CfScope, CfScopeKind};
use crate::{
  analyzer::Analyzer,
  entity::{Consumable, Entity, EntityDepNode},
};

#[derive(Debug, Default)]
pub struct ConditionalData<'a> {
  pub historical_indeterminate: bool,
  pub determinate_tests: Vec<Entity<'a>>,
  pub referred: bool,
}

impl<'a> Analyzer<'a> {
  pub fn push_conditional_cf_scope(
    &mut self,
    data: &mut ConditionalData<'a>,
    test: Entity<'a>,
    historical_indeterminate: bool,
    current_indeterminate: bool,
  ) {
    let dep_node = EntityDepNode::from_data(data);
    let dep = if historical_indeterminate {
      if data.referred || self.is_referred(dep_node) {
        data.referred = true;
        self.consume(test);
        None
      } else {
        Some((Consumable::from(data.determinate_tests.clone()), test, dep_node).into())
      }
    } else {
      data.determinate_tests.push(test);
      Some(dep_node.into())
    };
    self.scope_context.cf.push(CfScope::new(
      CfScopeKind::Conditional,
      None,
      dep,
      if current_indeterminate { None } else { Some(false) },
    ));
  }
}
