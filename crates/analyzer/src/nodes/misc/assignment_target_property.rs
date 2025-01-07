use crate::{host::Host, 
  analyzer::Analyzer,  dep::DepId,
};
use oxc::{
  ast::ast::{
    AssignmentTargetProperty, AssignmentTargetPropertyIdentifier, AssignmentTargetPropertyProperty,
  },
  span::{GetSpan, SPAN},
};

#[derive(Debug, Default)]
struct IdentifierData {
  need_init: bool,
}

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  /// Returns the key
  pub fn exec_assignment_target_property(
    &mut self,
    node: &'a AssignmentTargetProperty<'a>,
    value: H::Entity,
  ) -> H::Entity {
    let dep = self.consumable(DepId::from(AstKind2::AssignmentTargetProperty(node)));
    match node {
      AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(node) => {
        let key = self.factory.string(node.binding.name.as_str());

        let value = value.get_property(self, dep, key);

        let (need_init, value) = if let Some(init) = &node.init {
          self.exec_with_default(init, value)
        } else {
          (false, value)
        };

        let data =
          self.load_data::<IdentifierData>(AstKind2::AssignmentTargetPropertyIdentifier(node));
        data.need_init |= need_init;

        self.exec_identifier_reference_write(&node.binding, value);

        key
      }
      AssignmentTargetProperty::AssignmentTargetPropertyProperty(node) => {
        self.push_dependent_cf_scope(value);
        let key = self.exec_property_key(&node.name);
        self.pop_cf_scope();

        let value = value.get_property(self, dep, key);
        self.exec_assignment_target_maybe_default(&node.binding, value);
        key
      }
    }
  }
}

