use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::ast::{ArrayAssignmentTarget, AssignmentTargetPattern, ObjectAssignmentTarget};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_assignment_target_pattern_write(
    &mut self,
    node: &'a AssignmentTargetPattern<'a>,
    value: H::Entity,
  ) {
    match node {
      AssignmentTargetPattern::ArrayAssignmentTarget(node) => {
        let (element_values, rest_value, dep) = value.destruct_as_array(
          self,
          self.consumable(AstKind2::ArrayAssignmentTarget(node)),
          node.elements.len(),
          node.rest.is_some(),
        );

        self.push_dependent_cf_scope(dep);
        for (element, value) in node.elements.iter().zip(element_values) {
          if let Some(element) = element {
            self.exec_assignment_target_maybe_default(element, value);
          }
        }
        if let Some(rest) = &node.rest {
          self.exec_assignment_target_rest(rest, rest_value.unwrap());
        }
        self.pop_cf_scope();
      }
      AssignmentTargetPattern::ObjectAssignmentTarget(node) => {
        self.push_dependent_cf_scope(value.get_destructable(self, self.factory.empty_consumable));

        let is_nullish = value.test_nullish();
        if is_nullish != Some(false) {
          if is_nullish == Some(true) {
            self.thrown_builtin_error("Cannot destructure nullish value");
          } else {
            self.may_throw();
          }
          value.consume(self);
          self.refer_dep(AstKind2::ObjectAssignmentTarget(node));
        }

        let mut enumerated = vec![];
        for property in &node.properties {
          enumerated.push(self.exec_assignment_target_property(property, value));
        }
        if let Some(rest) = &node.rest {
          let dep = AstKind2::ObjectAssignmentTarget(node);
          let init = self.exec_object_rest(dep, value, enumerated);
          self.exec_assignment_target_rest(rest, init);
        }

        self.pop_cf_scope();
      }
    }
  }
}

