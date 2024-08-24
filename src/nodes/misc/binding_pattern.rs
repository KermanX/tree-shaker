use std::rc::Rc;

use crate::{
  analyzer,
  entity::{
    dep::EntityDep,
    entity::{Entity, EntityTrait},
    literal::LiteralEntity,
    union::UnionEntity,
  },
  transformer::Transformer,
  Analyzer,
};
use oxc::{
  ast::ast::{
    ArrayPattern, BindingIdentifier, BindingPattern, BindingPatternKind, BindingProperty,
    ObjectPattern, TSTypeAnnotation,
  },
  span::GetSpan,
};

#[derive(Debug)]
struct BindingIdentifierEntity<'a> {
  val: Entity<'a>,
  dep: EntityDep<'a>,
}

impl<'a> EntityTrait<'a> for BindingIdentifierEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {
    self.val.consume_self(analyzer);
    analyzer.refer_dep(self.dep);
  }

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    self.val.consume_as_unknown(analyzer);
    analyzer.refer_dep(self.dep);
  }

  fn consume_as_array(
    &self,
    analyzer: &mut Analyzer<'a>,
    length: usize,
  ) -> (Vec<Entity<'a>>, Entity<'a>) {
    analyzer.refer_dep(self.dep);
    self.val.consume_as_array(analyzer, length)
  }

  fn test_truthy(&self) -> Option<bool> {
    self.val.test_truthy()
  }

  fn test_nullish(&self) -> Option<bool> {
    self.val.test_nullish()
  }

  fn get_literal(&self) -> Option<LiteralEntity<'a>> {
    self.val.get_literal()
  }

  fn get_property(&self, key: &Entity<'a>) -> Entity<'a> {
    self.val.get_property(key)
  }

  fn call(&self, analyzer: &mut Analyzer<'a>, this: &Entity<'a>, args: &Entity<'a>) -> Entity<'a> {
    self.val.call(analyzer, this, args)
  }
}

impl<'a> BindingIdentifierEntity<'a> {
  pub(crate) fn new(val: Entity<'a>, dep: EntityDep<'a>) -> Entity<'a> {
    Rc::new(Self { val, dep })
  }
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_binding_pattern(&mut self, node: &'a BindingPattern<'a>, init: Entity<'a>) {
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        let symbol = node.symbol_id.get().unwrap();
        self.declare_symbol(
          symbol,
          BindingIdentifierEntity::new(init, EntityDep::BindingIdentifier(node)),
        );
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
          None => self.exec_indeterminate(|s| {
            UnionEntity::new(vec![s.exec_expression(&node.right), init.clone()])
          }),
        };
      }
    }
  }

  // pub(crate) fn calc_binding_pattern(
  //   &self,
  //   node: &'a BindingPattern<'a>,
  //   symbol: SymbolId,
  // ) -> Option<Entity> {
  //   let data = self.get_data::<Data>(AST_TYPE, node);

  //   match &node.kind {
  //     BindingPatternKind::BindingIdentifier(node) => {
  //       (node.symbol_id.get().unwrap() == symbol).then(|| data.init_val.clone())
  //     }
  //     BindingPatternKind::ObjectPattern(node) => {
  //       for property in &node.properties {
  //         if let Some(val) = self.calc_binding_pattern(&property.value, symbol) {
  //           return Some(val);
  //         }
  //       }
  //       node.rest.as_ref().and_then(|rest| self.calc_binding_rest_element(rest, symbol))
  //     }
  //     BindingPatternKind::ArrayPattern(node) => {
  //       for element in &node.elements {
  //         if let Some(element) = element {
  //           if let Some(val) = self.calc_binding_pattern(&element, symbol) {
  //             return Some(val);
  //           }
  //         }
  //       }
  //       node.rest.as_ref().and_then(|rest| self.calc_binding_rest_element(rest, symbol))
  //     }
  //     BindingPatternKind::AssignmentPattern(node) => self.calc_binding_pattern(&node.left, symbol),
  //   }
  // }

  // pub(crate) fn refer_binding_pattern(&mut self, node: &'a BindingPattern, symbol: SymbolId) {
  //   let data = self.load_data::<Data>(AST_TYPE, node);

  //   match &node.kind {
  //     BindingPatternKind::BindingIdentifier(node) => {
  //       data.referred |= node.symbol_id.get().unwrap() == symbol;
  //     }
  //     BindingPatternKind::ObjectPattern(node) => {
  //       for property in &node.properties {
  //         self.refer_binding_pattern(&property.value, symbol);
  //       }
  //       node.rest.as_ref().map(|rest| self.refer_binding_rest_element(rest, symbol));
  //     }
  //     BindingPatternKind::ArrayPattern(node) => {
  //       for element in &node.elements {
  //         if let Some(element) = element {
  //           self.refer_binding_pattern(&element, symbol);
  //         }
  //       }
  //       node.rest.as_ref().map(|rest| self.refer_binding_rest_element(rest, symbol));
  //     }
  //     BindingPatternKind::AssignmentPattern(node) => {
  //       self.refer_binding_pattern(&node.left, symbol);
  //     }
  //   }
  // }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_binding_pattern(
    &self,
    node: BindingPattern<'a>,
  ) -> Option<BindingPattern<'a>> {
    let BindingPattern { kind, .. } = node;

    match kind {
      BindingPatternKind::BindingIdentifier(node) => {
        if self.is_referred(EntityDep::BindingIdentifier(&node)) {
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
