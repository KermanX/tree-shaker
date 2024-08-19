use crate::{entity::Entity, TreeShakerImpl};
use oxc::{
  ast::ast::{BindingPattern, BindingPatternKind},
  semantic::SymbolId,
  syntax::symbol,
};
use rustc_hash::FxHashSet;

#[derive(Debug, Default, Clone)]
pub struct Data {
  included_symbols: FxHashSet<SymbolId>,
}

impl<'a> TreeShakerImpl<'a> {
  pub(crate) fn exec_binding_pattern(
    &mut self,
    node: &'a BindingPattern,
    need_symbol: Option<SymbolId>,
    init_val: Entity,
  ) -> Option<Entity> {
    let data = self.load_data::<Data>(node);
    need_symbol.map(|symbol| data.included_symbols.insert(symbol));

    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        let symbol_id = node.symbol_id.get().unwrap();
        self.declare_symbol(symbol_id);
        need_symbol.and_then(|s| if symbol_id == s { Some(init_val) } else { None })
      }
      BindingPatternKind::ObjectPattern(node) => {
        let mut result: Option<Entity> = None;
        for property in &node.properties {
          if need_symbol.is_some_and(|s| self.is_in_binding_pattern(&property.value, s)) {
            let key = self.exec_property_key(&property.key);
            let value = init_val.get_property(&key).as_ref().clone();
            result = self.exec_binding_pattern(&property.value, need_symbol, value)
          } else {
            self.exec_property_key(&property.key);
            self.exec_binding_pattern(&property.value, None, Entity::Unknown);
          }
        }
        // TODO: rest property
        result
      }
      BindingPatternKind::ArrayPattern(node) => {
        let mut result: Option<Entity> = None;
        for (index, property) in node.elements.iter().enumerate() {
          if let Some(property) = property {
            if need_symbol.is_some_and(|s| self.is_in_binding_pattern(&property, s)) {
              let key = Entity::NumberLiteral(index as f64);
              let value = init_val.get_property(&key).as_ref().clone();
              result = self.exec_binding_pattern(&property, need_symbol, value)
            } else {
              self.exec_binding_pattern(&property, None, Entity::Unknown);
            }
          }
        }
        // TODO: rest property
        result
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let value = self.exec_binding_pattern(&node.left, need_symbol, init_val);
        if value.as_ref().is_some_and(|value| value.is_null_or_undefined()) {
          self.exec_expression(&node.right);
        }
        value
      }
      _ => todo!(),
    }
  }

  fn is_in_binding_pattern(&self, node: &'a BindingPattern, symbol_id: SymbolId) -> bool {
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => node.symbol_id.get().unwrap() == symbol_id,
      BindingPatternKind::ObjectPattern(node) => {
        for property in &node.properties {
          if self.is_in_binding_pattern(&property.value, symbol_id) {
            return true;
          }
        }
        false
      }
      BindingPatternKind::ArrayPattern(node) => {
        for element in &node.elements {
          if let Some(element) = element {
            if self.is_in_binding_pattern(element, symbol_id) {
              return true;
            }
          }
        }
        false
      }
      BindingPatternKind::AssignmentPattern(node) => {
        self.is_in_binding_pattern(&node.left, symbol_id)
      }
    }
  }
}
