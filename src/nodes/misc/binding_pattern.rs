use crate::{
  ast::{AstType2, DeclarationKind},
  entity::entity::Entity,
  transformer::Transformer,
  Analyzer,
};
use oxc::{
  ast::ast::{
    ArrayPattern, AssignmentPattern, BindingPattern, BindingPatternKind, BindingProperty,
    ObjectPattern, TSTypeAnnotation,
  },
  span::GetSpan,
};

#[derive(Debug, Default)]
struct Data {
  has_effect: bool,
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

  /// effect_and_init is a tuple of (effect, init)
  /// effect is a boolean value that indicates whether the binding pattern has an effect:
  /// ```js
  /// const { a } = { get a() { effect() }};
  /// ```
  /// here `a` has an effect
  pub fn exec_binding_pattern(
    &mut self,
    node: &'a BindingPattern<'a>,
    (effect, init): (bool, Entity<'a>),
  ) {
    if effect {
      let data = self.load_data::<Data>(AstType2::BindingPattern, node);
      data.has_effect = true;
    }
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        self.init_binding_identifier(node, init);
      }
      BindingPatternKind::ObjectPattern(node) => {
        let mut enumerated = vec![];
        for property in &node.properties {
          let key = self.exec_property_key(&property.key);
          enumerated.push(key.clone());
          let effect_and_init = init.get_property(self, &key);
          self.exec_binding_pattern(&property.value, effect_and_init);
        }
        if let Some(rest) = &node.rest {
          let effect_and_init = self.exec_object_rest(init, enumerated);
          self.init_binding_rest_element(rest, effect_and_init);
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        let (element_values, rest_value) = init.get_to_array(node.elements.len());
        for (element, value) in node.elements.iter().zip(element_values) {
          if let Some(element) = element {
            self.exec_binding_pattern(element, (false, value));
          }
        }
        if let Some(rest) = &node.rest {
          self.init_binding_rest_element(rest, (false, rest_value));
        }
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let (need_right, binding_val) = self.exec_with_default(&node.right, init);

        let data =
          self.load_data::<AssignmentPatternData>(AstType2::AssignmentPattern, node.as_ref());
        data.need_right |= need_right;

        self.exec_binding_pattern(&node.left, (false, binding_val));
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_binding_pattern(
    &self,
    node: &'a BindingPattern<'a>,
  ) -> Option<BindingPattern<'a>> {
    let data = self.get_data::<Data>(AstType2::BindingPattern, node);

    let span = node.span();

    let BindingPattern { kind, .. } = node;

    let transformed = match kind {
      BindingPatternKind::BindingIdentifier(node) => {
        self.transform_binding_identifier(node).map(|identifier| {
          self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_from_binding_identifier(identifier),
            None::<TSTypeAnnotation>,
            false,
          )
        })
      }
      BindingPatternKind::ObjectPattern(node) => {
        let ObjectPattern { span, properties, rest, .. } = node.as_ref();

        let rest = rest.as_ref().and_then(|rest| self.transform_binding_rest_element(rest));

        let mut transformed_properties = self.ast_builder.vec();
        for property in properties {
          let BindingProperty { span, key, value, shorthand, .. } = property;
          let key_span = key.span();
          let value = self.transform_binding_pattern(value);
          if let Some(value) = value {
            let (computed, key) = self.transform_property_key(key, true).unwrap();
            transformed_properties
              .push(self.ast_builder.binding_property(*span, key, value, *shorthand, computed));
          } else if let Some((computed, key)) = self.transform_property_key(key, rest.is_some()) {
            transformed_properties.push(self.ast_builder.binding_property(
              *span,
              key,
              self.build_unused_binding_pattern(key_span),
              *shorthand,
              computed,
            ));
          }
        }
        if transformed_properties.is_empty() && rest.is_none() {
          None
        } else {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_object_pattern(
              *span,
              transformed_properties,
              rest,
            ),
            None::<TSTypeAnnotation>,
            false,
          ))
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        let ArrayPattern { span, elements, rest, .. } = node.as_ref();

        let mut transformed_elements = self.ast_builder.vec();
        for element in elements {
          transformed_elements
            .push(element.as_ref().and_then(|element| self.transform_binding_pattern(element)));
        }

        let rest = rest.as_ref().and_then(|rest| self.transform_binding_rest_element(rest));

        while transformed_elements.last().is_none() {
          transformed_elements.pop();
        }

        if transformed_elements.is_empty() && rest.is_none() {
          None
        } else {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_array_pattern(*span, transformed_elements, rest),
            None::<TSTypeAnnotation>,
            false,
          ))
        }
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let data =
          self.get_data::<AssignmentPatternData>(AstType2::AssignmentPattern, node.as_ref());

        let AssignmentPattern { span, left, right, .. } = node.as_ref();

        let left_span = left.span();
        let left = self.transform_binding_pattern(left);
        let right =
          data.need_right.then(|| self.transform_expression(right, left.is_some())).flatten();

        if let Some(right) = right {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_assignment_pattern(
              *span,
              left.unwrap_or(self.build_unused_binding_pattern(left_span)),
              right,
            ),
            None::<TSTypeAnnotation>,
            false,
          ))
        } else {
          left
        }
      }
    };

    if data.has_effect {
      Some(transformed.unwrap_or_else(|| self.build_unused_binding_pattern(span)))
    } else {
      transformed
    }
  }
}
