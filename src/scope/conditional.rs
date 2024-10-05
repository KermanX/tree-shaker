use super::{cf_scope::CfScope, CfScopeKind};
use crate::{
  analyzer::Analyzer,
  entity::{Entity, EntityDepNode},
};

#[derive(Debug, Default)]
pub struct ConditionalData<'a> {
  pub determinate_tests: Vec<Entity<'a>>,
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
        data.determinate_tests.push(test);
        for val in &data.determinate_tests {
          deps.push(val.clone().into());
        }
        deps.push(dep_node.into());
        deps
      }
    } else {
      data.determinate_tests.push(test);
      vec![dep_node.into()]
    };
    self.scope_context.cf.push(CfScope::new(
      kind,
      None,
      deps,
      if current_indeterminate { None } else { Some(false) },
    ));
  }
}
