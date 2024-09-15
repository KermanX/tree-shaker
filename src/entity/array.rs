use super::{
  consumed_object,
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  entry::EntryEntity,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  union::UnionEntity,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use crate::{analyzer::Analyzer, scope::cf_scope::CfScopes, use_consumed_flag};
use oxc::syntax::number::ToJsInt32;
use std::cell::{Cell, RefCell};

#[derive(Debug)]
pub struct ArrayEntity<'a> {
  consumed: Cell<bool>,
  cf_scopes: CfScopes<'a>,
  pub elements: RefCell<Vec<Entity<'a>>>,
  pub rest: RefCell<Option<Entity<'a>>>,
}

impl<'a> EntityTrait<'a> for ArrayEntity<'a> {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);
    for element in self.elements.borrow().iter() {
      element.consume_as_unknown(analyzer);
    }
    if let Some(rest) = self.rest.borrow().as_ref() {
      rest.consume_as_unknown(analyzer);
    }
  }

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_property(analyzer, dep, key);
    }
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let mut result = vec![];
      let mut rest_added = false;
      let mut undefined_added = false;
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
                if !undefined_added {
                  undefined_added = true;
                  result.push(LiteralEntity::new_undefined());
                }
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
            } else if let Some(property) = analyzer.builtins.prototypes.array.get(key) {
              result.push(property.clone());
            } else if !undefined_added {
              undefined_added = true;
              result.push(LiteralEntity::new_undefined());
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!(),
        }
      }
      EntryEntity::new(UnionEntity::new(result), key.clone())
    } else {
      UnknownEntity::new_unknown()
    }
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    if self.consumed.get() {
      return consumed_object::set_property(analyzer, dep, key, value);
    }
    let indeterminate = analyzer.is_assignment_indeterminate(&self.cf_scopes);
    let mut has_effect = false;
    if let Some(key_literals) = key.get_to_property_key().get_to_literals() {
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
                  has_effect = true;
                }
              } else {
                has_effect = true;
              }
            } else {
              self.consume_as_unknown(analyzer);
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!(),
        }
      }
      todo!("{has_effect:?}")
    } else {
      self.consume_as_unknown(analyzer);
      consumed_object::set_property(analyzer, dep, key, value)
    }
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    if self.consumed.get() {
      return consumed_object::enumerate_properties(analyzer, dep);
    }
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
    entries
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
    if self.consumed.get() {
      return consumed_object::delete_property(analyzer, key);
    }
    // TODO: delete array element
    self.consume_as_unknown(analyzer);
    key.get_to_property_key().consume_self(analyzer);
    true
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.consume_as_unknown(analyzer);
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(&self, rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    if self.consumed.get() {
      return consumed_object::r#await(analyzer);
    }
    // FIXME: additional `then` method?
    (false, rc.clone())
  }

  fn iterate(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Option<Entity<'a>>) {
    if self.consumed.get() {
      return consumed_object::iterate(analyzer);
    }
    let elements = self.elements.borrow();
    (false, if elements.is_empty() { None } else { Some(UnionEntity::new(elements.clone())) })
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("object")
  }

  fn get_to_string(&self, rc: &Entity<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_string();
    }
    UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![rc.clone()])
  }

  fn get_to_property_key(&self, rc: &Entity<'a>) -> Entity<'a> {
    self.get_to_string(rc)
  }

  fn get_to_array(&self, _rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    if self.consumed.get() {
      return consumed_object::get_to_array(length);
    }
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
      consumed: Cell::new(false),
      cf_scopes: self.scope_context.cf_scopes.clone(),
      elements: RefCell::new(Vec::new()),
      rest: RefCell::new(None),
    }
  }
}
