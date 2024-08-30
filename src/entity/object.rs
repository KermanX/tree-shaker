use super::{
  arguments::ArgumentsEntity,
  entity::{Entity, EntityTrait},
  entry::EntryEntity,
  function::FunctionEntity,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::{UnknownEntity, UnknownEntityKind},
  utils::collect_effect_and_value,
};
use crate::analyzer::Analyzer;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, Default)]
pub(crate) struct ObjectEntity<'a> {
  string_keyed: RefCell<FxHashMap<&'a str, ObjectProperty<'a>>>,
  unknown_keyed: RefCell<ObjectProperty<'a>>,
  // TODO: symbol_keyed
  rest: RefCell<ObjectProperty<'a>>,
}

#[derive(Debug, Clone)]
enum ObjectPropertyKind<'a> {
  Field(Entity<'a>),
  /// (Getter, Setter)
  Property(Option<FunctionEntity<'a>>, Option<FunctionEntity<'a>>),
}

type ObjectProperty<'a> = Vec<ObjectPropertyKind<'a>>;

impl<'a> ObjectPropertyKind<'a> {
  pub fn get_from_vec(
    analyzer: &mut Analyzer<'a>,
    vec: &ObjectProperty<'a>,
    this: &Entity<'a>,
  ) -> Vec<(bool, Entity<'a>)> {
    let mut values = Vec::new();
    for property in vec {
      values.push(match property {
        ObjectPropertyKind::Field(value) => (false, value.clone()),
        ObjectPropertyKind::Property(Some(getter), _) => {
          getter.call(analyzer, this, &ArgumentsEntity::new(vec![]))
        }
        _ => (false, LiteralEntity::new_undefined()),
      });
    }
    values
  }
}

impl<'a> EntityTrait<'a> for ObjectEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {}

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    fn consume_property_as_unknown<'a>(property: &ObjectProperty<'a>, analyzer: &mut Analyzer<'a>) {
      for value in property {
        match value {
          ObjectPropertyKind::Field(value) => value.consume_as_unknown(analyzer),
          ObjectPropertyKind::Property(getter, setter) => {
            getter.as_ref().map(|f| f.consume_as_unknown(analyzer));
            setter.as_ref().map(|f| f.consume_as_unknown(analyzer));
          }
        }
      }
    }

    for property in self.string_keyed.borrow().values() {
      consume_property_as_unknown(property, analyzer);
    }
    consume_property_as_unknown(&self.rest.borrow(), analyzer);
    consume_property_as_unknown(&self.unknown_keyed.borrow(), analyzer);
  }

  fn get_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> (bool, Entity<'a>) {
    let this = UnknownEntity::new_unknown(); // TODO: this
    let key = key.get_to_property_key();
    let string_keyed = self.string_keyed.borrow();
    if let Some(key_literals) = key.get_to_literals() {
      let mut values =
        ObjectPropertyKind::get_from_vec(analyzer, self.unknown_keyed.borrow().as_ref(), &this);
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(value) = string_keyed.get(key) {
              values.extend(ObjectPropertyKind::get_from_vec(analyzer, value, &this));
            } else {
              todo!("rest");
            }
          }
          _ => todo!("rest"),
        }
      }
      let (has_effect, value) = collect_effect_and_value(values);
      (has_effect, EntryEntity::new(value, key.clone()))
    } else {
      (true, EntryEntity::new(UnknownEntity::new_unknown(), key.clone()))
    }
  }

  fn set_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool {
    todo!()
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
  pub(crate) fn init_field(&self, key: Entity<'a>, value: Entity<'a>) {
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let determinate = key_literals.len() == 1;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let mut string_keyed = self.string_keyed.borrow_mut();
            let property = ObjectPropertyKind::Field(value.clone());
            let existing = string_keyed.get_mut(key);
            if determinate || existing.is_none() {
              string_keyed.insert(key, vec![property]);
            } else {
              existing.unwrap().push(property);
            }
          }
          LiteralEntity::Symbol(key) => todo!(),
          _ => unreachable!(),
        }
      }
    } else {
      self.unknown_keyed.borrow_mut().push(ObjectPropertyKind::Field(EntryEntity::new(value, key)));
    }
  }

  pub(crate) fn init_spread(&mut self, argument: Entity<'a>) {
    todo!()
  }
}
