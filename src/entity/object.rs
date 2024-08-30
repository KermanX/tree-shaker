use super::{
  entity::{Entity, EntityTrait},
  entry::EntryEntity,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  union::UnionEntity,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use crate::analyzer::Analyzer;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub(crate) struct ObjectEntity<'a> {
  string_keyed: RefCell<FxHashMap<&'a str, Entity<'a>>>,
  // TODO: symbol_keyed
  rest: RefCell<Option<Entity<'a>>>,
  common: RefCell<Vec<Entity<'a>>>,
}

impl<'a> EntityTrait<'a> for ObjectEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {}

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    for (_, value) in self.string_keyed.borrow().iter() {
      value.consume_self(analyzer);
    }
    if let Some(rest) = self.rest.borrow().as_ref() {
      rest.consume_self(analyzer);
    }
  }

  fn get_property(&self, key: &Entity<'a>) -> (bool, Entity<'a>) {
    let key = key.get_to_property_key();
    let string_keyed = self.string_keyed.borrow();
    if let Some(key_literals) = key.get_to_literals() {
      let mut has_effect = false;
      let mut values = self.common.borrow().clone();
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(value) = string_keyed.get(key) {
              // TODO: getter call + effect
              values.push(value.clone());
            } else {
              has_effect = true;
              todo!("rest");
            }
          }
          _ => todo!("rest"),
        }
      }
      (has_effect, EntryEntity::new(UnionEntity::new(values), key.clone()))
    } else {
      (true, EntryEntity::new(UnknownEntity::new_unknown(), key.clone()))
    }
  }

  fn set_property(&self, key: &Entity<'a>, value: Entity<'a>) -> bool {
    let key = key.get_to_property_key();
    let mut string_keyed = self.string_keyed.borrow_mut();
    let mut common = self.common.borrow_mut();
    if let Some(key_literals) = key.get_to_literals() {
      let mut has_effect = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            // TODO: setter call + effect
            string_keyed.insert(key, value.clone());
          }
          _ => {
            has_effect = true;
            common.push(EntryEntity::new(value.clone(), key.clone()));
          }
        }
      }
      has_effect
    } else {
      common.push(EntryEntity::new(value.clone(), key.clone()));
      true
    }
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("object")
  }

  fn get_to_string(&self) -> Entity<'a> {
    UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![Rc::new(self.clone())])
  }

  fn get_to_property_key(&self) -> Entity<'a> {
    self.get_to_string()
  }

  fn get_to_array(&self, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    todo!()
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

impl<'a> ObjectEntity<'a> {
  pub(crate) fn init_property(&self, key: Entity<'a>, value: Entity<'a>) {
    let key = key.get_to_property_key();
    let mut string_keyed = self.string_keyed.borrow_mut();
    if let Some(key_literals) = key.get_to_literals() {
      let determinate = key_literals.len() == 1;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if determinate {
              string_keyed.insert(key, value.clone());
            } else {
              let existing =
                string_keyed.get(key).map_or_else(|| LiteralEntity::new_undefined(), Rc::clone);
              let union = UnionEntity::new(vec![existing, value.clone()]);
              string_keyed.insert(key, union);
            }
          }
          _ => {
            // self.common.push(ForwardedEntity::new(value.clone(), key.clone()));
          }
        }
      }
    } else {
      // self.common.push(ForwardedEntity::new(value.clone(), key.clone()));
    }
  }

  pub(crate) fn init_spread(&mut self, argument: Entity<'a>) {
    todo!()
  }
}

impl<'a> ObjectEntity<'a> {
  pub(crate) fn new_empty() -> ObjectEntity<'a> {
    Self {
      string_keyed: RefCell::new(FxHashMap::default()),
      rest: RefCell::new(None),
      common: RefCell::new(vec![]),
    }
  }
}
