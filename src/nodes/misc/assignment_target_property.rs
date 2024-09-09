use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{entity::Entity, literal::LiteralEntity, union::UnionEntity},
  transformer::Transformer,
};
use oxc::{
  ast::ast::{
    AssignmentTargetProperty, AssignmentTargetPropertyIdentifier, AssignmentTargetPropertyProperty,
  },
  span::GetSpan,
};

#[derive(Debug, Default)]
struct IdentifierData {
  need_init: bool,
}

impl<'a> Analyzer<'a> {
  /// Returns the key
  pub fn exec_assignment_target_property(
    &mut self,
    node: &'a AssignmentTargetProperty<'a>,
    value: Entity<'a>,
  ) -> Entity<'a> {
    match node {
      AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(node) => {
        let (need_init, value) = if let Some(init) = &node.init {
          self.exec_with_default(init, value)
        } else {
          (false, value)
        };

        let data = self
          .load_data::<IdentifierData>(AstType2::AssignmentTargetPropertyIdentifier, node.as_ref());
        data.need_init |= need_init;

        self.exec_identifier_reference_write(&node.binding, value);

        LiteralEntity::new_string(node.binding.name.as_str())
      }
      AssignmentTargetProperty::AssignmentTargetPropertyProperty(node) => {
        let key = self.exec_property_key(&node.name);
        let effect_and_value = value.get_property(self, &key);
        self.exec_assignment_target_maybe_default(&node.binding, effect_and_value);
        key
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_assignment_target_property(
    &self,
    node: &'a AssignmentTargetProperty<'a>,
    has_rest: bool,
  ) -> Option<AssignmentTargetProperty<'a>> {
    match node {
      AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(node) => {
        let data = self
          .get_data::<IdentifierData>(AstType2::AssignmentTargetPropertyIdentifier, node.as_ref());

        let AssignmentTargetPropertyIdentifier { span, binding, init, .. } = node.as_ref();

        let binding_span = binding.span();
        let binding = self.transform_identifier_reference_write(binding);
        let init = data
          .need_init
          .then(|| {
            init.as_ref().and_then(|init| self.transform_expression(init, binding.is_some()))
          })
          .flatten();

        if binding.is_some() || init.is_some() {
          Some(self.ast_builder.assignment_target_property_assignment_target_property_identifier(
            *span,
            binding.unwrap_or(self.build_unused_identifier_reference_write(binding_span)),
            init,
          ))
        } else {
          None
        }
      }
      AssignmentTargetProperty::AssignmentTargetPropertyProperty(node) => {
        let AssignmentTargetPropertyProperty { span, name, binding, .. } = node.as_ref();

        let name_span = name.span();
        let binding = self.transform_assignment_target_maybe_default(binding);
        if let Some(binding) = binding {
          let (_computed, name) = self.transform_property_key(name, true).unwrap();
          Some(
            self
              .ast_builder
              .assignment_target_property_assignment_target_property_property(*span, name, binding),
          )
        } else if let Some((_computed, name)) = self.transform_property_key(name, has_rest) {
          Some(self.ast_builder.assignment_target_property_assignment_target_property_property(
            *span,
            name,
            self.ast_builder.assignment_target_maybe_default_assignment_target(
              self.build_unused_assignment_target(name_span),
            ),
          ))
        } else {
          None
        }
      }
    }
  }
}
