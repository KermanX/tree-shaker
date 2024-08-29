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
use std::rc::Rc;

#[derive(Debug, Default, Clone)]
pub(crate) struct ObjectEntity<'a> {
  string_keyed: FxHashMap<&'a str, Entity<'a>>,
  // TODO: symbol_keyed
  rest: Option<Entity<'a>>,
  common: Vec<Entity<'a>>,
}

impl<'a> EntityTrait<'a> for ObjectEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {}

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    for (_, value) in self.string_keyed.iter() {
      value.consume_self(analyzer);
    }
    if let Some(rest) = &self.rest {
      rest.consume_self(analyzer);
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

  fn get_property(&self, key: &Entity<'a>) -> Entity<'a> {
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let mut values = self.common.clone();
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(value) = self.string_keyed.get(key) {
              values.push(value.clone());
            } else {
              todo!("rest");
            }
          }
          _ => todo!("rest"),
        }
      }
      EntryEntity::new(UnionEntity::new(values), key.clone())
    } else {
      EntryEntity::new(UnknownEntity::new_unknown(), key.clone())
    }
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
  pub(crate) fn set_property(&mut self, key: Entity<'a>, value: Entity<'a>) {
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let determinate = key_literals.len() == 1;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let existing = self.string_keyed.get(key);
            if determinate || existing.is_none() {
              self.string_keyed.insert(key, value.clone());
            } else {
              let existing = existing.unwrap();
              let union = UnionEntity::new(vec![existing.clone(), value.clone()]);
              self.string_keyed.insert(key, union);
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

  pub(crate) fn set_spread(&mut self, argument: Entity<'a>) {
    todo!()
  }
}
