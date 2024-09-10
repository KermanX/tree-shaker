use crate::{analyzer::Analyzer, entity::entity::Entity, transformer::Transformer};
use oxc::ast::ast::{ArrayAssignmentTarget, AssignmentTargetPattern, ObjectAssignmentTarget};

impl<'a> Analyzer<'a> {
  pub fn exec_assignment_target_pattern(
    &mut self,
    node: &'a AssignmentTargetPattern<'a>,
    value: Entity<'a>,
  ) {
    match node {
      AssignmentTargetPattern::ArrayAssignmentTarget(node) => {
        let (element_values, rest_value) = value.get_to_array(node.elements.len());
        for (element, value) in node.elements.iter().zip(element_values) {
          if let Some(element) = element {
            self.exec_assignment_target_maybe_default(element, (false, value));
          }
        }
        if let Some(rest) = &node.rest {
          self.exec_assignment_target_rest(rest, (false, rest_value));
        }
      }
      AssignmentTargetPattern::ObjectAssignmentTarget(node) => {
        let mut enumerated = vec![];
        for property in &node.properties {
          enumerated.push(self.exec_assignment_target_property(property, value.clone()));
        }
        if let Some(rest) = &node.rest {
          let effect_and_value = self.exec_object_rest(value, enumerated);
          self.exec_assignment_target_rest(rest, effect_and_value);
        }
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_assignment_target_pattern(
    &self,
    node: &'a AssignmentTargetPattern<'a>,
  ) -> Option<AssignmentTargetPattern<'a>> {
    match node {
      AssignmentTargetPattern::ArrayAssignmentTarget(node) => {
        let ArrayAssignmentTarget { span, elements, rest, .. } = node.as_ref();

        let mut transformed_elements = self.ast_builder.vec();
        for element in elements {
          transformed_elements.push(
            element
              .as_ref()
              .and_then(|element| self.transform_assignment_target_maybe_default(element)),
          );
        }

        let rest = rest.as_ref().and_then(|rest| self.transform_assignment_target_rest(rest));

        while transformed_elements.last().is_none() {
          transformed_elements.pop();
        }

        if transformed_elements.is_empty() && rest.is_none() {
          None
        } else {
          Some(self.ast_builder.assignment_target_pattern_array_assignment_target(
            *span,
            transformed_elements,
            rest,
            None,
          ))
        }
      }
      AssignmentTargetPattern::ObjectAssignmentTarget(node) => {
        let ObjectAssignmentTarget { span, properties, rest, .. } = node.as_ref();

        let rest = rest.as_ref().and_then(|rest| self.transform_assignment_target_rest(rest));

        let mut transformed_properties = self.ast_builder.vec();
        for property in properties {
          if let Some(property) =
            self.transform_assignment_target_property(property, rest.is_some())
          {
            transformed_properties.push(property);
          }
        }
        if transformed_properties.is_empty() && rest.is_none() {
          None
        } else {
          Some(self.ast_builder.assignment_target_pattern_object_assignment_target(
            *span,
            transformed_properties,
            rest,
          ))
        }
      }
    }
  }
}
