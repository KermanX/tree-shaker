use super::CfScopeKind;
use crate::{
  analyzer::Analyzer,
  entity::{Consumable, Entity, EntityDepNode},
};

#[derive(Debug, Default)]
pub struct ConditionalData<'a> {
  pub determinate_tests: Vec<Consumable<'a>>,
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
        let mut deps = vec![];
        data.determinate_tests.push(test.into());
        for val in &data.determinate_tests {
          deps.push(val.clone());
        }
        deps.push(dep_node.into());
        deps
      }
    } else {
      data.determinate_tests.push(test.into());
      vec![dep_node.into()]
    };
    self.push_cf_scope_with_deps(
      kind,
      None,
      deps,
      if current_indeterminate { None } else { Some(false) },
    );
  }
}
