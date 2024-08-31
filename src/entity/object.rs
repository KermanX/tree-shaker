use super::{
  arguments::ArgumentsEntity,
  entity::{Entity, EntityTrait},
  entry::EntryEntity,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::{UnknownEntity, UnknownEntityKind},
  utils::collect_effect_and_value,
};
use crate::analyzer::Analyzer;
use oxc::ast::ast::PropertyKind;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub(crate) struct ObjectEntity<'a> {
  string_keyed: RefCell<FxHashMap<&'a str, ObjectPropertyUnion<'a>>>,
  unknown_keyed: RefCell<ObjectPropertyUnion<'a>>,
  // TODO: symbol_keyed
  rest: RefCell<ObjectPropertyUnion<'a>>,
}

#[derive(Debug, Clone)]
enum ObjectProperty<'a> {
  Field(Entity<'a>),
  /// (Getter, Setter)
  Property(Option<Entity<'a>>, Option<Entity<'a>>),
}

impl<'a> ObjectProperty<'a> {
  pub fn get_value(&self, analyzer: &mut Analyzer<'a>, this: &Entity<'a>) -> (bool, Entity<'a>) {
    match self {
      ObjectProperty::Field(value) => (false, value.clone()),
      ObjectProperty::Property(Some(getter), _) => {
        getter.call(analyzer, this, &ArgumentsEntity::new(vec![]))
      }
      _ => (false, LiteralEntity::new_undefined()),
    }
  }

  pub fn get_value_from_vec(
    analyzer: &mut Analyzer<'a>,
    vec: &ObjectPropertyUnion<'a>,
    this: &Entity<'a>,
  ) -> Vec<(bool, Entity<'a>)> {
    vec.iter().map(|property| property.get_value(analyzer, this)).collect()
  }
}

type ObjectPropertyUnion<'a> = Vec<ObjectProperty<'a>>;

impl<'a> EntityTrait<'a> for ObjectEntity<'a> {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    fn consume_property_as_unknown<'a>(
      property: &ObjectPropertyUnion<'a>,
      analyzer: &mut Analyzer<'a>,
    ) {
      for value in property {
        match value {
          ObjectProperty::Field(value) => value.consume_as_unknown(analyzer),
          ObjectProperty::Property(getter, setter) => {
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
    let this = self.get_this();
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let mut values =
        ObjectProperty::get_value_from_vec(analyzer, self.unknown_keyed.borrow().as_ref(), &this);
      let mut rest_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let string_keyed = self.string_keyed.borrow();
            if let Some(value) = string_keyed.get(key) {
              values.extend(ObjectProperty::get_value_from_vec(analyzer, value, &this));
            } else if !rest_added {
              rest_added = true;
              let rest = self.rest.borrow();
              values.extend(ObjectProperty::get_value_from_vec(analyzer, &rest, &this));
              values.push((false, LiteralEntity::new_undefined()));
            }
          }
          LiteralEntity::Symbol(_) => todo!(),
          _ => unreachable!(),
        }
      }
      let (has_effect, value) = collect_effect_and_value(values);
      (has_effect, EntryEntity::new(value, key.clone()))
    } else {
      (true, EntryEntity::new(UnknownEntity::new_unknown(), key.clone()))
    }
  }

  fn set_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool {
    let this = self.get_this();
    let indeterminate = analyzer.cf_scope().is_indeterminate();
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let mut has_effect = false;
      let definite = key_literals.len() == 1;
      let indeterminate = indeterminate || self.unknown_keyed.borrow().len() > 0;
      let mut rest_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(properties) = self.string_keyed.borrow_mut().get_mut(key) {
              let indeterminate = indeterminate || properties.len() > 1;
              if definite {
                *properties = properties
                  .iter()
                  .filter(|v| matches!(v, ObjectProperty::Property(_, _)))
                  .cloned()
                  .collect::<Vec<_>>();
              }
              for property in properties.iter().chain(self.unknown_keyed.borrow().iter()) {
                if let ObjectProperty::Property(_, Some(setter)) = property {
                  has_effect |= setter
                    .call(analyzer, &this, &ArgumentsEntity::new(vec![(false, value.clone())]))
                    .0;
                }
              }
              if indeterminate || properties.is_empty() {
                properties.push(ObjectProperty::Field(value.clone()));
              }
            } else if !rest_added {
              rest_added = true;
              let mut rest = self.rest.borrow_mut();
              for property in rest.iter().chain(self.unknown_keyed.borrow().iter()) {
                if let ObjectProperty::Property(_, Some(setter)) = property {
                  has_effect |= setter
                    .call(analyzer, &this, &ArgumentsEntity::new(vec![(false, value.clone())]))
                    .0;
                }
              }
              rest.push(ObjectProperty::Field(value.clone()));
            }
          }
          LiteralEntity::Symbol(_) => todo!(),
          _ => unreachable!(),
        }
      }
      has_effect
    } else {
      self.unknown_keyed.borrow_mut().push(ObjectProperty::Field(EntryEntity::new(value, key)));
      self.apply_unknown_to_possible_setters(analyzer)
    }
  }

  fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(Entity<'a>, Entity<'a>)>) {
    let this = self.get_this();
    // unknown_keyed = unknown_keyed + rest
    let mut unknown_keyed =
      ObjectProperty::get_value_from_vec(analyzer, &self.unknown_keyed.borrow(), &this);
    unknown_keyed.extend(ObjectProperty::get_value_from_vec(analyzer, &self.rest.borrow(), &this));
    let mut has_effect = false;
    let mut result = Vec::new();
    if unknown_keyed.len() > 0 {
      let (effect, value) = collect_effect_and_value(unknown_keyed);
      has_effect |= effect;
      result.push((UnknownEntity::new_unknown(), value));
    }
    for (key, properties) in self.string_keyed.borrow().iter() {
      let values = ObjectProperty::get_value_from_vec(analyzer, properties, &this);
      let (effect, value) = collect_effect_and_value(values);
      has_effect |= effect;
      result.push((LiteralEntity::new_string(key), value));
    }
    (has_effect, result)
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
  pub(crate) fn new_empty_object() -> Self {
    Self {
      string_keyed: RefCell::new(FxHashMap::default()),
      unknown_keyed: RefCell::new(vec![]),
      rest: RefCell::new(vec![]),
    }
  }

  pub(crate) fn init_property(&self, kind: PropertyKind, key: Entity<'a>, value: Entity<'a>) {
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let definite = key_literals.len() == 1;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let mut string_keyed = self.string_keyed.borrow_mut();
            let existing = string_keyed.get_mut(key);
            let reused_property = definite
              .then(|| {
                existing.and_then(|existing| {
                  for property in existing.iter() {
                    match property {
                      ObjectProperty::Property(getter, setter) => {
                        return Some((getter.clone(), setter.clone()));
                      }
                      _ => {}
                    }
                  }
                  None
                })
              })
              .flatten();
            let property = match kind {
              PropertyKind::Init => ObjectProperty::Field(value.clone()),
              PropertyKind::Get => ObjectProperty::Property(
                Some(value.clone()),
                reused_property.and_then(|(_, setter)| setter),
              ),
              PropertyKind::Set => ObjectProperty::Property(
                reused_property.and_then(|(getter, _)| getter),
                Some(value.clone()),
              ),
            };
            let existing = string_keyed.get_mut(key);
            if definite || existing.is_none() {
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
      self.unknown_keyed.borrow_mut().push(ObjectProperty::Field(EntryEntity::new(value, key)));
    }
  }

  pub(crate) fn init_spread(&mut self, analyzer: &mut Analyzer<'a>, argument: Entity<'a>) -> bool {
    let (has_effect, properties) = argument.enumerate_properties(analyzer);
    for (key, value) in properties {
      self.init_property(PropertyKind::Init, key.clone(), value);
    }
    has_effect
  }

  fn get_this(&self) -> Entity<'a> {
    UnknownEntity::new_unknown() // TODO: handle `this`
  }

  fn apply_unknown_to_possible_setters(&self, analyzer: &mut Analyzer<'a>) -> bool {
    fn apply_unknown_to_vec<'a>(
      analyzer: &mut Analyzer<'a>,
      vec: &ObjectPropertyUnion<'a>,
      this: &Entity<'a>,
    ) -> bool {
      let mut has_effect = false;
      for property in vec {
        if let ObjectProperty::Property(_, Some(setter)) = property {
          has_effect |= setter
            .call(
              analyzer,
              this,
              &ArgumentsEntity::new(vec![(false, UnknownEntity::new_unknown())]),
            )
            .0;
        }
      }
      has_effect
    }

    let mut has_effect = false;
    for property in self.string_keyed.borrow().values() {
      has_effect |= apply_unknown_to_vec(analyzer, property, &UnknownEntity::new_unknown());
    }
    has_effect |= apply_unknown_to_vec(
      analyzer,
      &mut self.unknown_keyed.borrow(),
      &UnknownEntity::new_unknown(),
    );
    has_effect |=
      apply_unknown_to_vec(analyzer, &mut self.rest.borrow(), &UnknownEntity::new_unknown());
    has_effect
  }
}
