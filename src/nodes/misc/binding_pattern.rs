use crate::{
  ast::{AstType2, DeclarationKind},
  consumable::box_consumable,
  entity::{Entity, EntityDepNode},
  transformer::Transformer,
  Analyzer,
};
use oxc::{
  ast::{
    ast::{
      ArrayPattern, AssignmentPattern, BindingPattern, BindingPatternKind, BindingProperty,
      ObjectPattern,
    },
    AstKind, NONE,
  },
  span::GetSpan,
};

#[derive(Debug, Default)]
struct ObjectPatternData {
  need_destruct: bool,
}

#[derive(Debug, Default)]
struct AssignmentPatternData {
  need_right: bool,
}

impl<'a> Analyzer<'a> {
  pub fn declare_binding_pattern(
    &mut self,
    node: &'a BindingPattern<'a>,
    exporting: bool,
    kind: DeclarationKind,
  ) {
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        self.declare_binding_identifier(node, exporting, kind);
      }
      BindingPatternKind::ObjectPattern(node) => {
        for property in &node.properties {
          self.declare_binding_pattern(&property.value, exporting, kind);
        }
        if let Some(rest) = &node.rest {
          self.declare_binding_rest_element(rest, exporting, kind);
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        for element in &node.elements {
          if let Some(element) = element {
            self.declare_binding_pattern(element, exporting, kind);
          }
        }
        if let Some(rest) = &node.rest {
          self.declare_binding_rest_element(rest, exporting, kind);
        }
      }
      BindingPatternKind::AssignmentPattern(node) => {
        self.declare_binding_pattern(&node.left, exporting, kind);
      }
    }
  }

  pub fn init_binding_pattern(&mut self, node: &'a BindingPattern<'a>, init: Option<Entity<'a>>) {
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        self.init_binding_identifier(node, init);
      }
      BindingPatternKind::ObjectPattern(node) => {
        let init = init.unwrap_or_else(|| {
          self.thrown_builtin_error("Missing initializer in destructuring declaration");
          self.factory.unknown
        });

        let is_nullish = init.test_nullish();
        if is_nullish != Some(false) {
          if is_nullish == Some(true) {
            self.thrown_builtin_error("Cannot destructure nullish value");
          } else {
            self.may_throw();
          }
          init.consume(self);
          let data = self.load_data::<ObjectPatternData>(AstType2::ObjectPattern, node.as_ref());
          data.need_destruct = true;
        }

        let mut enumerated = vec![];
        for property in &node.properties {
          let dep = box_consumable(EntityDepNode::from((AstType2::BindingProperty, property)));

          self.push_cf_scope_for_dep(init.clone());
          let key = self.exec_property_key(&property.key);
          self.pop_cf_scope();

          enumerated.push(key.clone());
          let init = init.get_property(self, dep, key);
          self.init_binding_pattern(&property.value, Some(init));
        }
        if let Some(rest) = &node.rest {
          let dep = AstKind::BindingRestElement(rest.as_ref());
          let init = self.exec_object_rest(dep, init, enumerated);
          self.init_binding_rest_element(rest, init);
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        let init = init.unwrap_or_else(|| {
          self.thrown_builtin_error("Missing initializer in destructuring declaration");
          self.factory.unknown
        });

        let (element_values, rest_value) = init.destruct_as_array(
          self,
          box_consumable(AstKind::ArrayPattern(node)),
          node.elements.len(),
        );
        for (element, value) in node.elements.iter().zip(element_values) {
          if let Some(element) = element {
            self.init_binding_pattern(element, Some(value));
          }
        }
        if let Some(rest) = &node.rest {
          self.init_binding_rest_element(rest, rest_value);
        }
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let (need_right, binding_val) = self.exec_with_default(&node.right, init.unwrap());

        let data =
          self.load_data::<AssignmentPatternData>(AstType2::AssignmentPattern, node.as_ref());
        data.need_right |= need_right;

        self.init_binding_pattern(&node.left, Some(binding_val));
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_binding_pattern(
    &self,
    node: &'a BindingPattern<'a>,
    need_binding: bool,
  ) -> Option<BindingPattern<'a>> {
    let span = node.span();

    let BindingPattern { kind, .. } = node;

    let transformed = match kind {
      BindingPatternKind::BindingIdentifier(node) => {
        let result = self.transform_binding_identifier(node).map(|identifier| {
          self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_from_binding_identifier(identifier),
            NONE,
            false,
          )
        });

        if need_binding {
          Some(result.unwrap_or_else(|| self.build_unused_binding_identifier(span)))
        } else {
          result
        }
      }
      BindingPatternKind::ObjectPattern(node) => {
        let ObjectPattern { span, properties, rest, .. } = node.as_ref();

        let data = self.get_data::<ObjectPatternData>(AstType2::ObjectPattern, node.as_ref());

        let rest = rest.as_ref().and_then(|rest| {
          self.transform_binding_rest_element(
            rest,
            self.is_referred(AstKind::BindingRestElement(rest.as_ref())),
          )
        });

        let mut transformed_properties = self.ast_builder.vec();
        for property in properties {
          let dep = (AstType2::BindingProperty, property);
          let need_property = rest.is_some() || self.is_referred(dep);

          let BindingProperty { span, key, value, shorthand, .. } = property;

          if *shorthand && matches!(value.kind, BindingPatternKind::BindingIdentifier(_)) {
            if need_property
              || self.transform_property_key(key, false).is_some()
              || self.transform_binding_pattern(value, false).is_some()
            {
              transformed_properties.push(self.clone_node(property));
            }
          } else {
            let transformed_key = self.transform_property_key(key, need_property);
            let value = self.transform_binding_pattern(value, transformed_key.is_some());
            if let Some(value) = value {
              let (computed, key) =
                transformed_key.unwrap_or_else(|| self.transform_property_key(key, true).unwrap());
              transformed_properties
                .push(self.ast_builder.binding_property(*span, key, value, *shorthand, computed));
            }
          }
        }

        if !need_binding
          && transformed_properties.is_empty()
          && rest.is_none()
          && !data.need_destruct
        {
          None
        } else {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_object_pattern(
              *span,
              transformed_properties,
              rest,
            ),
            NONE,
            false,
          ))
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        let ArrayPattern { span, elements, rest, .. } = node.as_ref();

        let is_referred = self.is_referred(AstKind::ArrayPattern(node));

        let mut transformed_elements = self.ast_builder.vec();
        for element in elements {
          transformed_elements.push(
            element.as_ref().and_then(|element| self.transform_binding_pattern(element, false)),
          );
        }

        let rest =
          rest.as_ref().and_then(|rest| self.transform_binding_rest_element(rest, is_referred));

        if !is_referred && rest.is_none() {
          while transformed_elements.last().is_some_and(Option::is_none) {
            transformed_elements.pop();
          }
        }

        if !need_binding && !is_referred && transformed_elements.is_empty() && rest.is_none() {
          None
        } else {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_array_pattern(*span, transformed_elements, rest),
            NONE,
            false,
          ))
        }
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let data =
          self.get_data::<AssignmentPatternData>(AstType2::AssignmentPattern, node.as_ref());

        let AssignmentPattern { span, left, right, .. } = node.as_ref();

        let left_span = left.span();
        let transformed_left = self.transform_binding_pattern(left, false);
        let transformed_right = if self.declaration_only.get() {
          None
        } else {
          data
            .need_right
            .then(|| self.transform_expression(right, transformed_left.is_some()))
            .flatten()
        };

        if let Some(right) = transformed_right {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_assignment_pattern(
              *span,
              transformed_left.unwrap_or(self.build_unused_binding_pattern(left_span)),
              right,
            ),
            NONE,
            false,
          ))
        } else if need_binding {
          Some(
            transformed_left.unwrap_or_else(|| self.transform_binding_pattern(left, true).unwrap()),
          )
        } else {
          transformed_left
        }
      }
    };

    transformed
  }
}
