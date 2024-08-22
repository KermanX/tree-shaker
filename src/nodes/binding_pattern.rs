use std::rc::Rc;

use crate::{entity::Entity, symbol::SymbolSource, transformer::Transformer, Analyzer};
use oxc::{
  ast::ast::{
    ArrayPattern, AssignmentPattern, BindingPattern, BindingPatternKind, BindingProperty,
    BindingRestElement, FormalParameter, ObjectPattern, TSTypeAnnotation, VariableDeclarator,
  },
  semantic::SymbolId,
  span::GetSpan,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum BindingPatternSource<'a> {
  VariableDeclarator(&'a VariableDeclarator<'a>),
  FormalParameter(&'a FormalParameter<'a>),
  BindingRestElement(&'a BindingRestElement<'a>),
}

impl<'a> BindingPatternSource<'a> {
  pub(self) fn to_symble_source(&self, symbol: SymbolId) -> SymbolSource<'a> {
    match self {
      BindingPatternSource::VariableDeclarator(node) => {
        SymbolSource::VariableDeclarator(node, symbol)
      }
      BindingPatternSource::FormalParameter(node) => SymbolSource::FormalParameter(node, symbol),
      BindingPatternSource::BindingRestElement(node) => {
        SymbolSource::BindingRestElement(node, symbol)
      }
    }
  }
}

#[derive(Debug, Default, Clone)]
pub struct Data {
  init_val: Entity,
  referred: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_binding_pattern(
    &mut self,
    node: &'a BindingPattern<'a>,
    source: BindingPatternSource<'a>,
    init_val: Entity,
  ) -> bool {
    let mut effect = false;
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        let symbol = node.symbol_id.get().unwrap();
        self.declare_symbol(source.to_symble_source(symbol), symbol);
      }
      BindingPatternKind::ObjectPattern(node) => {
        for property in &node.properties {
          let (key_effect, key_val) = self.exec_property_key(&property.key);
          effect |= key_effect;
          effect |= self.exec_binding_pattern(
            &property.value,
            source,
            (*init_val.get_property(&key_val)).clone(),
          );
        }
        // TODO: rest
      }
      BindingPatternKind::ArrayPattern(node) => {
        for (index, element) in node.elements.iter().enumerate() {
          if let Some(element) = element {
            let key_val = Entity::StringLiteral(index.to_string());
            effect |= self.exec_binding_pattern(
              element,
              source,
              (*init_val.get_property(&key_val)).clone(),
            );
          }
        }
        // TODO: rest
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let is_nullable = init_val.is_null_or_undefined();
        let binding_val = match is_nullable {
          Some(true) => self.calc_expression(&node.right),
          Some(false) => init_val.clone(),
          None => Entity::Union(vec![
            Rc::new(self.calc_expression(&node.right)),
            Rc::new(init_val.clone()),
          ])
          .simplify(),
        };
        effect |= self.exec_binding_pattern(&node.left, source, binding_val);
        effect |= match is_nullable {
          Some(true) => self.exec_expression(&node.right).0,
          Some(false) => false,
          None => {
            let backup = self.start_indeterminate();
            let (right_effect, _) = self.exec_expression(&node.right);
            self.end_indeterminate(backup);
            right_effect
          }
        };
      }
    }

    self.set_data(node, Data { init_val, referred: false });

    effect
  }

  pub(crate) fn calc_binding_pattern(
    &self,
    node: &'a BindingPattern<'a>,
    symbol: SymbolId,
  ) -> Option<Entity> {
    let data = self.get_data::<Data>(node);

    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        (node.symbol_id.get().unwrap() == symbol).then(|| data.init_val.clone())
      }
      BindingPatternKind::ObjectPattern(node) => {
        for property in &node.properties {
          if let Some(val) = self.calc_binding_pattern(&property.value, symbol) {
            return Some(val);
          }
        }
        node.rest.as_ref().and_then(|rest| self.calc_binding_rest_element(rest, symbol))
      }
      BindingPatternKind::ArrayPattern(node) => {
        for element in &node.elements {
          if let Some(element) = element {
            if let Some(val) = self.calc_binding_pattern(&element, symbol) {
              return Some(val);
            }
          }
        }
        node.rest.as_ref().and_then(|rest| self.calc_binding_rest_element(rest, symbol))
      }
      BindingPatternKind::AssignmentPattern(node) => self.calc_binding_pattern(&node.left, symbol),
    }
  }

  pub(crate) fn refer_binding_pattern(&mut self, node: &'a BindingPattern, symbol: SymbolId) {
    let data = self.load_data::<Data>(node);

    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        data.referred |= node.symbol_id.get().unwrap() == symbol;
      }
      BindingPatternKind::ObjectPattern(node) => {
        for property in &node.properties {
          self.refer_binding_pattern(&property.value, symbol);
        }
        node.rest.as_ref().map(|rest| self.refer_binding_rest_element(rest, symbol));
      }
      BindingPatternKind::ArrayPattern(node) => {
        for (index, element) in node.elements.iter().enumerate() {
          if let Some(element) = element {
            self.refer_binding_pattern(&element, symbol);
          }
        }
        node.rest.as_ref().map(|rest| self.refer_binding_rest_element(rest, symbol));
      }
      BindingPatternKind::AssignmentPattern(node) => {
        self.refer_binding_pattern(&node.left, symbol);
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_binding_pattern(
    &self,
    node: BindingPattern<'a>,
  ) -> Option<BindingPattern<'a>> {
    let data = self.get_data::<Data>(&node);

    let BindingPattern { kind, .. } = node;

    match kind {
      BindingPatternKind::BindingIdentifier(node) => {
        if data.referred {
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
          let BindingProperty { span, key, value, shorthand, computed, .. } = property;
          let key_span = key.span();
          let value = self.transform_binding_pattern(value);
          if let Some(value) = value {
            transformed_properties.push(self.ast_builder.binding_property(
              span,
              self.transform_property_key(key, true).unwrap(),
              value,
              shorthand,
              computed,
            ));
          } else if let Some(key) = self.transform_property_key(key, false) {
            transformed_properties.push(self.ast_builder.binding_property(
              span,
              key,
              self.new_unused_binding_pattern(key_span),
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
        let AssignmentPattern { span, left, right, .. } = node.unbox();
        let left_span = left.span();
        let left: Option<BindingPattern> = self.transform_binding_pattern(left);
        let right = match data.init_val.is_null_or_undefined() {
          Some(false) => None,
          _ => self.transform_expression(right, left.is_some()),
        };
        if let Some(right) = right {
          Some(self.ast_builder.binding_pattern(
            self.ast_builder.binding_pattern_kind_assignment_pattern(
              span,
              left.unwrap_or(self.new_unused_binding_pattern(left_span)),
              right,
            ),
            None::<TSTypeAnnotation>,
            false,
          ))
        } else {
          left
        }
      }
    }
  }
}
