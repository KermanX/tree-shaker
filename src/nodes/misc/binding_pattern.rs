use crate::{
  entity::{
    dep::{EntityDep, EntityDepNode},
    entity::Entity,
    forwarded::ForwardedEntity,
    literal::LiteralEntity,
    union::UnionEntity,
  },
  transformer::Transformer,
  Analyzer,
};
use oxc::{
  ast::ast::{
    ArrayPattern, BindingPattern, BindingPatternKind, BindingProperty, ObjectPattern,
    TSTypeAnnotation,
  },
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_binding_pattern(&mut self, node: &'a BindingPattern<'a>, init: Entity<'a>) {
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        let symbol = node.symbol_id.get().unwrap();
        let dep = self.new_entity_dep(EntityDepNode::BindingIdentifier(node));
        self.declare_symbol(symbol, ForwardedEntity::new(init, vec![dep]));
      }
      BindingPatternKind::ObjectPattern(node) => {
        for property in &node.properties {
          let key = self.exec_property_key(&property.key);
          self.exec_binding_pattern(&property.value, init.get_property(&key));
        }
        if let Some(rest) = &node.rest {
          self.exec_binding_rest_element(rest, todo!());
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        for (index, element) in node.elements.iter().enumerate() {
          if let Some(element) = element {
            let key = LiteralEntity::new_string(self.allocator.alloc(index.to_string()).as_str());
            // FIXME: get_property !== iterate
            self.exec_binding_pattern(element, init.get_property(&key));
          }
        }
        if let Some(rest) = &node.rest {
          self.exec_binding_rest_element(rest, todo!());
        }
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let is_nullable = init.test_nullish();
        let binding_val = match is_nullable {
          Some(true) => self.exec_expression(&node.right),
          Some(false) => init.clone(),
          None => {
            self.push_indeterminate_scope(true);
            let value = UnionEntity::new(vec![self.exec_expression(&node.right), init.clone()]);
            self.pop_indeterminate_scope();
            value
          }
        };
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_binding_pattern(
    &mut self,
    node: BindingPattern<'a>,
  ) -> Option<BindingPattern<'a>> {
    let BindingPattern { kind, .. } = node;

    match kind {
      BindingPatternKind::BindingIdentifier(node) => {
        if self.is_referred(EntityDepNode::BindingIdentifier(&node)) {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_from_binding_identifier(node),
            None::<TSTypeAnnotation>,
            false,
          ))
        } else {
          None
        }
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
        // let AssignmentPattern { span, left, right, .. } = node.unbox();
        // let left_span = left.span();
        // let left: Option<BindingPattern> = self.transform_binding_pattern(left);
        // let right = match data.init_val.is_null_or_undefined() {
        //   Some(false) => None,
        //   _ => self.transform_expression(right, left.is_some()),
        // };
        // if let Some(right) = right {
        //   Some(self.ast_builder.binding_pattern(
        //     self.ast_builder.binding_pattern_kind_assignment_pattern(
        //       span,
        //       left.unwrap_or(self.build_unused_binding_pattern(left_span)),
        //       right,
        //     ),
        //     None::<TSTypeAnnotation>,
        //     false,
        //   ))
        // } else {
        //   left
        // }
        todo!("p4")
      }
    }
  }
}
