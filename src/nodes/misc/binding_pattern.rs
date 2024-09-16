use crate::{
  ast::{AstType2, DeclarationKind},
  entity::{dep::EntityDepNode, entity::Entity},
  transformer::Transformer,
  Analyzer,
};
use oxc::{
  ast::{
    ast::{
      ArrayPattern, AssignmentPattern, BindingPattern, BindingPatternKind, BindingProperty,
      BindingRestElement, ObjectPattern, TSTypeAnnotation,
    },
    AstKind,
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

  pub fn init_binding_pattern(&mut self, node: &'a BindingPattern<'a>, init: Entity<'a>) {
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        self.init_binding_identifier(node, init);
      }
      BindingPatternKind::ObjectPattern(node) => {
        if init.test_nullish() != Some(false) {
          self.may_throw();
          let data = self.load_data::<ObjectPatternData>(AstType2::ObjectPattern, node.as_ref());
          data.need_destruct = true;
        }

        let mut enumerated = vec![];
        for property in &node.properties {
          let dep = EntityDepNode::from((AstType2::BindingProperty, property));
          let key = self.exec_property_key(&property.key);
          enumerated.push(key.clone());
          let init = init.get_property(self, dep, &key);
          self.init_binding_pattern(&property.value, init);
        }
        if let Some(rest) = &node.rest {
          let dep = AstKind::BindingRestElement(rest.as_ref());
          let init = self.exec_object_rest(dep, init, enumerated);
          self.init_binding_rest_element(rest, init);
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        let (element_values, rest_value) =
          init.destruct_as_array(self, AstKind::ArrayPattern(node), node.elements.len());
        for (element, value) in node.elements.iter().zip(element_values) {
          if let Some(element) = element {
            self.init_binding_pattern(element, value);
          }
        }
        if let Some(rest) = &node.rest {
          self.init_binding_rest_element(rest, rest_value);
        }
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let (need_right, binding_val) = self.exec_with_default(&node.right, init);

        let data =
          self.load_data::<AssignmentPatternData>(AstType2::AssignmentPattern, node.as_ref());
        data.need_right |= need_right;

        self.init_binding_pattern(&node.left, binding_val);
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

        let data = self.get_data::<ObjectPatternData>(AstType2::ObjectPattern, node.as_ref());

        let rest = rest.as_ref().and_then(|rest| {
          self.transform_binding_rest_element(
            rest,
            self.is_referred(AstKind::BindingRestElement(rest.as_ref())),
          )
        });

        let mut transformed_properties = self.ast_builder.vec();
        for property in properties {
          let BindingProperty { span, key, value, shorthand, .. } = property;
          let dep = (AstType2::BindingProperty, property);
          let transformed_key =
            self.transform_property_key(key, rest.is_some() || self.is_referred(dep));
          let value = self.transform_binding_pattern(value, transformed_key.is_some());
          if let Some(value) = value {
            let (computed, key) =
              transformed_key.unwrap_or_else(|| self.transform_property_key(key, true).unwrap());
            transformed_properties
              .push(self.ast_builder.binding_property(*span, key, value, *shorthand, computed));
          }
        }
        if transformed_properties.is_empty() && rest.is_none() && !data.need_destruct {
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
          transformed_elements.push(
            element.as_ref().and_then(|element| self.transform_binding_pattern(element, false)),
          );
        }

        let rest = rest.as_ref().and_then(|rest| self.transform_binding_rest_element(rest, false));

        while transformed_elements.last().is_none() {
          if transformed_elements.pop().is_none() {
            break;
          }
        }

        if transformed_elements.is_empty() && rest.is_none() {
          self.is_referred(AstKind::ArrayPattern(node)).then(|| {
            self.ast_builder.binding_pattern(
              self.ast_builder.binding_pattern_kind_array_pattern(
                *span,
                self.ast_builder.vec(),
                None::<BindingRestElement>,
              ),
              None::<TSTypeAnnotation>,
              false,
            )
          })
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
        let left = self.transform_binding_pattern(left, false);
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

    if need_binding {
      Some(transformed.unwrap_or_else(|| self.build_unused_binding_pattern(span)))
    } else {
      transformed
    }
  }
}
