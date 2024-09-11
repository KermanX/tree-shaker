use super::{
  entity::{Entity, EntityTrait},
  entry::EntryEntity,
  label::LabelEntity,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  union::UnionEntity,
  unknown::{UnknownEntity, UnknownEntityKind},
  utils::is_assignment_indeterminate,
};
use crate::analyzer::Analyzer;
use oxc::{semantic::ScopeId, syntax::number::ToJsInt32};
use std::cell::RefCell;

#[derive(Debug)]
pub struct ArrayEntity<'a> {
  pub scope_path: Vec<ScopeId>,
  pub elements: RefCell<Vec<Entity<'a>>>,
  pub rest: RefCell<Option<Entity<'a>>>,
}

impl<'a> EntityTrait<'a> for ArrayEntity<'a> {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    for element in self.elements.borrow().iter() {
      element.consume_as_unknown(analyzer);
    }
    if let Some(rest) = self.rest.borrow().as_ref() {
      rest.consume_as_unknown(analyzer);
    }
  }

  fn get_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> (bool, Entity<'a>) {
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let mut result = vec![];
      let mut rest_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(index) = key.parse::<usize>().ok() {
              if let Some(element) = self.elements.borrow().get(index) {
                result.push(element.clone());
              } else if !rest_added {
                rest_added = true;
                if let Some(rest) = self.rest.borrow().as_ref() {
                  result.push(rest.clone());
                }
                result.push(LiteralEntity::new_undefined());
              }
            } else if key == "length" {
              result.push(self.get_length().map_or_else(
                || {
                  UnknownEntity::new_with_deps(
                    UnknownEntityKind::Number,
                    vec![self.rest.borrow().as_ref().unwrap().clone()],
                  )
                },
                |length| {
                  LiteralEntity::new_number(
                    (length as f64).into(),
                    analyzer.allocator.alloc(length.to_string()),
                  )
                },
              ));
            } else {
              todo!("builtins {:?}", key);
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!(),
        }
      }
      (false, EntryEntity::new(UnionEntity::new(result), key.clone()))
    } else {
      (false, UnknownEntity::new_unknown())
    }
  }

  fn set_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool {
    let indeterminate = is_assignment_indeterminate(&self.scope_path, analyzer);
    let key = key.get_to_property_key();
    let mut has_effect = false;
    if let Some(key_literals) = key.get_to_literals() {
      let definite = !indeterminate && key_literals.len() == 1;
      let mut rest_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(index) = key.parse::<usize>().ok() {
              if let Some(element) = self.elements.borrow_mut().get_mut(index) {
                *element = if definite {
                  value.clone()
                } else {
                  UnionEntity::new(vec![element.clone(), value.clone()])
                };
              } else if !rest_added {
                rest_added = true;
                if let Some(rest) = self.rest.borrow_mut().as_mut() {
                  *rest = if definite {
                    value.clone()
                  } else {
                    UnionEntity::new(vec![rest.clone(), value.clone()])
                  };
                } else {
                  *self.rest.borrow_mut() = Some(value.clone());
                }
              }
            } else if key == "length" {
              if let Some(length) = value.get_literal().and_then(|lit| lit.to_number()) {
                if let Some(length) = length.map(|l| l.0.to_js_int_32()) {
                  let length = length as usize;
                  let mut elements = self.elements.borrow_mut();
                  let mut rest = self.rest.borrow_mut();
                  if elements.len() > length {
                    has_effect = true;
                    elements.truncate(length);
                    *rest = None;
                  } else if let Some(rest) = rest.as_mut() {
                    has_effect = true;
                    *rest = UnionEntity::new(vec![rest.clone(), LiteralEntity::new_undefined()]);
                  } else if elements.len() < length {
                    has_effect = true;
                    for _ in elements.len()..length {
                      elements.push(LiteralEntity::new_undefined());
                    }
                  }
                } else {
                  // TODO: throw warning: Invalid array length
                }
              }
            } else {
              todo!("builtins");
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!(),
        }
      }
      has_effect
    } else {
      self.consume_as_unknown(analyzer);
      true
    }
  }

  fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
    let mut entries = Vec::new();
    for (i, element) in self.elements.borrow().iter().enumerate() {
      entries.push((
        true,
        LiteralEntity::new_string(analyzer.allocator.alloc(i.to_string())),
        element.clone(),
      ));
    }
    if let Some(rest) = self.rest.borrow().as_ref() {
      entries.push((true, UnknownEntity::new(UnknownEntityKind::String), rest.clone()));
    }
    (false, entries)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, _key: &Entity<'a>) -> bool {
    self.consume_as_unknown(analyzer);
    true
  }

  fn call(
    &self,
    _analyzer: &mut Analyzer<'a>,
    _this: &Entity<'a>,
    _args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    // TODO: throw warning
    (true, UnknownEntity::new_unknown())
  }

  fn r#await(&self, rc: &Entity<'a>, _analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    // FIXME: additional `then` method?
    (false, rc.clone())
  }

  fn iterate(&self, _rc: &Entity<'a>, _analyzer: &mut Analyzer<'a>) -> (bool, Option<Entity<'a>>) {
    let elements = self.elements.borrow();
    (false, if elements.is_empty() { None } else { Some(UnionEntity::new(elements.clone())) })
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("object")
  }

  fn get_to_string(&self, rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![rc.clone()])
  }

  fn get_to_property_key(&self, rc: &Entity<'a>) -> Entity<'a> {
    self.get_to_string(rc)
  }

  fn get_to_array(&self, _rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    let elements = self.elements.borrow();
    let mut result = Vec::new();
    for i in 0..length.min(elements.len()) {
      result.push(elements[i].clone());
    }
    for _ in 0..length.saturating_sub(elements.len()) {
      result.push(UnknownEntity::new_unknown());
    }
    (result, UnknownEntity::new_unknown())
  }

  fn test_typeof(&self) -> TypeofResult {
    TypeofResult::Object
  }

  fn test_truthy(&self) -> Option<bool> {
    Some(true)
  }

  fn test_nullish(&self) -> Option<bool> {
    Some(false)
  }

  fn test_is_array(&self) -> Option<bool> {
    Some(false)
  }
}

impl<'a> ArrayEntity<'a> {
  pub fn push_element(&self, element: Entity<'a>) {
    self.elements.borrow_mut().push(element);
  }

  pub fn init_rest(&self, rest: Entity<'a>) {
    *self.rest.borrow_mut() = Some(rest);
  }

  pub fn get_length(&self) -> Option<usize> {
    if self.rest.borrow().is_some() {
      None
    } else {
      Some(self.elements.borrow().len())
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_empty_array(&self) -> ArrayEntity<'a> {
    ArrayEntity {
      scope_path: self.variable_scope_path(),
      elements: RefCell::new(Vec::new()),
      rest: RefCell::new(None),
    }
  }
}
