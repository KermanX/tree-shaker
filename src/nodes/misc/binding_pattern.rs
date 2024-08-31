use crate::{
  ast::AstType2,
  entity::{
    dep::EntityDepNode, entity::Entity, forwarded::ForwardedEntity, literal::LiteralEntity,
    union::UnionEntity,
  },
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
  /// effect_and_init is a tuple of (effect, init)
  /// effect is a boolean value that indicates whether the binding pattern has an effect:
  /// ```js
  /// const { a } = { get a() { effect() }};
  /// ```
  /// here `a` has an effect
  pub(crate) fn exec_binding_pattern(
    &mut self,
    node: &'a BindingPattern<'a>,
    effect_and_init: (bool, Entity<'a>),
    exporting: bool,
  ) {
    let (effect, init) = effect_and_init;
    if effect {
      let data = self.load_data::<Data>(AstType2::BindingPattern, node);
      data.has_effect = true;
    }
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        let symbol = node.symbol_id.get().unwrap();
        let dep = self.new_entity_dep(EntityDepNode::BindingIdentifier(node));
        self.declare_symbol(symbol, dep.clone(), ForwardedEntity::new(init, dep), exporting);
      }
      BindingPatternKind::ObjectPattern(node) => {
        let has_rest = node.rest.is_some();
        let mut enumerated_keys = vec![];
        for property in &node.properties {
          let key = self.exec_property_key(&property.key);
          enumerated_keys.push(key.clone());
          let (effect, init) = init.get_property(self, &key);
          self.exec_binding_pattern(&property.value, (effect || has_rest, init), exporting);
        }
        if let Some(rest) = &node.rest {
          self.exec_binding_rest_element_from_obj(rest, init, exporting, enumerated_keys);
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        let has_rest = node.rest.is_some();
        for (index, element) in node.elements.iter().enumerate() {
          if let Some(element) = element {
            let key = LiteralEntity::new_string(self.allocator.alloc(index.to_string()).as_str());
            let (effect, init) = init.get_property(self, &key);
            // FIXME: get_property !== iterate
            self.exec_binding_pattern(element, (effect || has_rest, init), exporting);
          }
        }
        if let Some(rest) = &node.rest {
          self.exec_binding_rest_element_from_arr(rest, init, exporting);
        }
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let is_undefined = init.test_is_undefined();
        let binding_val = match is_undefined {
          Some(true) => self.exec_expression(&node.right),
          Some(false) => init,
          None => {
            self.push_cf_scope(None);
            let value = UnionEntity::new(vec![self.exec_expression(&node.right), init]);
            self.pop_cf_scope();
            value
          }
        };
        self.exec_binding_pattern(&node.left, (false, binding_val), exporting);

        let data =
          self.load_data::<AssignmentPatternData>(AstType2::AssignmentPattern, node.as_ref());
        data.need_right |= !matches!(is_undefined, Some(false));
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_binding_pattern(
    &mut self,
    node: BindingPattern<'a>,
  ) -> Option<BindingPattern<'a>> {
    let data = self.get_data::<Data>(AstType2::BindingPattern, &node);

    let span = node.span();

    let BindingPattern { kind, .. } = node;

    let transformed = match kind {
      BindingPatternKind::BindingIdentifier(node) => {
        let referred = self.is_referred(EntityDepNode::BindingIdentifier(&node));
        referred.then(|| {
          self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_from_binding_identifier(node),
            None::<TSTypeAnnotation>,
            false,
          )
        })
      }
      BindingPatternKind::ObjectPattern(node) => {
        let ObjectPattern { span, properties, rest, .. } = node.unbox();
        let mut transformed_properties = self.ast_builder.vec();
        for property in properties {
          let BindingProperty { span, key, value, shorthand, .. } = property;
          let key_span = key.span();
          let value = self.transform_binding_pattern(value);
          if let Some(value) = value {
            let (computed, key) = self.transform_property_key(key, true).unwrap();
            transformed_properties
              .push(self.ast_builder.binding_property(span, key, value, shorthand, computed));
          } else if let Some((computed, key)) = self.transform_property_key(key, false) {
            transformed_properties.push(self.ast_builder.binding_property(
              span,
              key,
              self.build_unused_binding_pattern(key_span),
              shorthand,
              computed,
            ));
          }
        }
        let rest = rest.and_then(|rest| self.transform_binding_rest_element(rest.unbox()));
        if transformed_properties.is_empty() && rest.is_none() {
          None
        } else {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_object_pattern(
              span,
              transformed_properties,
              rest,
            ),
            None::<TSTypeAnnotation>,
            false,
          ))
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        let ArrayPattern { span, elements, rest, .. } = node.unbox();
        let mut transformed_elements = self.ast_builder.vec();
        for element in elements {
          transformed_elements
            .push(element.and_then(|element| self.transform_binding_pattern(element)));
        }
        let rest = rest.and_then(|rest| self.transform_binding_rest_element(rest.unbox()));

        while transformed_elements.last().is_none() {
          transformed_elements.pop();
        }

        if transformed_elements.is_empty() && rest.is_none() {
          None
        } else {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_array_pattern(span, transformed_elements, rest),
            None::<TSTypeAnnotation>,
            false,
          ))
        }
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let data =
          self.get_data::<AssignmentPatternData>(AstType2::AssignmentPattern, node.as_ref());

        let AssignmentPattern { span, left, right, .. } = node.unbox();

        let left_span = left.span();
        let left = self.transform_binding_pattern(left);
        let right =
          data.need_right.then(|| self.transform_expression(right, left.is_some())).flatten();

        if let Some(right) = right {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_assignment_pattern(
              span,
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
