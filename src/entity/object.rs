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
use oxc::{ast::ast::PropertyKind, semantic::ScopeId};
use rustc_hash::FxHashMap;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct ObjectEntity<'a> {
  pub scope_path: Vec<ScopeId>,
  pub string_keyed: RefCell<FxHashMap<&'a str, ObjectProperty<'a>>>,
  pub unknown_keyed: RefCell<ObjectProperty<'a>>,
  // TODO: symbol_keyed
  pub rest: RefCell<ObjectProperty<'a>>,
}

#[derive(Debug, Clone)]
pub enum ObjectPropertyValue<'a> {
  Field(Entity<'a>),
  /// (Getter, Setter)
  Property(Option<Entity<'a>>, Option<Entity<'a>>),
}

impl<'a> ObjectPropertyValue<'a> {
  pub fn get_value(&self, analyzer: &mut Analyzer<'a>, this: &Entity<'a>) -> (bool, Entity<'a>) {
    match self {
      ObjectPropertyValue::Field(value) => (false, value.clone()),
      ObjectPropertyValue::Property(Some(getter), _) => {
        getter.call(analyzer, this, &ArgumentsEntity::new(vec![]))
      }
      _ => (false, LiteralEntity::new_undefined()),
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct ObjectProperty<'a> {
  pub definite: bool,
  pub values: Vec<ObjectPropertyValue<'a>>,
}

impl<'a> ObjectProperty<'a> {
  pub fn get_value(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
  ) -> Vec<(bool, Entity<'a>)> {
    self.values.iter().map(|property| property.get_value(analyzer, this)).collect()
  }
}

impl<'a> EntityTrait<'a> for ObjectEntity<'a> {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    fn consume_property_as_unknown<'a>(property: &ObjectProperty<'a>, analyzer: &mut Analyzer<'a>) {
      for value in &property.values {
        match value {
          ObjectPropertyValue::Field(value) => value.consume_as_unknown(analyzer),
          ObjectPropertyValue::Property(getter, setter) => {
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
      let mut values = self.unknown_keyed.borrow().get_value(analyzer, &this);
      let mut rest_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let string_keyed = self.string_keyed.borrow();
            if let Some(property) = string_keyed.get(key) {
              values.extend(property.get_value(analyzer, &this));
            } else if !rest_added {
              rest_added = true;
              let rest = self.rest.borrow();
              values.extend(rest.get_value(analyzer, &this));
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
    let indeterminate = self.is_assignment_indeterminate(analyzer);
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let mut has_effect = false;
      let definite = key_literals.len() == 1;
      let indeterminate = indeterminate || self.unknown_keyed.borrow().values.len() > 0;
      let mut rest_set = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(property) = self.string_keyed.borrow_mut().get_mut(key) {
              let indeterminate = indeterminate || !property.definite || property.values.len() > 1;
              if definite {
                property.values = property
                  .values
                  .iter()
                  .filter(|v| matches!(v, ObjectPropertyValue::Property(_, _)))
                  .cloned()
                  .collect::<Vec<_>>();
              }
              for property_val in
                property.values.iter().chain(self.unknown_keyed.borrow().values.iter())
              {
                if let ObjectPropertyValue::Property(_, Some(setter)) = property_val {
                  has_effect |= setter
                    .call(analyzer, &this, &ArgumentsEntity::new(vec![(false, value.clone())]))
                    .0;
                }
              }
              if indeterminate || property.values.is_empty() {
                property.values.push(ObjectPropertyValue::Field(value.clone()));
              }
            } else {
              // Call setters in rest and unknown_keyed
              if !rest_set {
                rest_set = true;
                let rest = self.rest.borrow_mut();
                for property in rest.values.iter().chain(self.unknown_keyed.borrow().values.iter())
                {
                  if let ObjectPropertyValue::Property(_, Some(setter)) = property {
                    has_effect |= setter
                      .call(analyzer, &this, &ArgumentsEntity::new(vec![(false, value.clone())]))
                      .0;
                  }
                }
              }

              let mut string_keyed = self.string_keyed.borrow_mut();
              let property = ObjectProperty {
                definite,
                values: vec![ObjectPropertyValue::Field(value.clone())],
              };
              string_keyed.insert(key, property);
            }
          }
          LiteralEntity::Symbol(_) => todo!(),
          _ => unreachable!(),
        }
      }
      has_effect
    } else {
      self
        .unknown_keyed
        .borrow_mut()
        .values
        .push(ObjectPropertyValue::Field(EntryEntity::new(value, key)));
      self.apply_unknown_to_possible_setters(analyzer)
    }
  }

  fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
    let this = self.get_this();
    // unknown_keyed = unknown_keyed + rest
    let mut unknown_keyed = self.unknown_keyed.borrow().get_value(analyzer, &this);
    unknown_keyed.extend(self.rest.borrow().get_value(analyzer, &this));
    let mut has_effect = false;
    let mut result = Vec::new();
    if unknown_keyed.len() > 0 {
      let (effect, value) = collect_effect_and_value(unknown_keyed);
      has_effect |= effect;
      result.push((false, UnknownEntity::new_unknown(), value));
    }
    for (key, properties) in self.string_keyed.borrow().iter() {
      let values = properties.get_value(analyzer, &this);
      let (effect, value) = collect_effect_and_value(values);
      has_effect |= effect;
      result.push((properties.definite, LiteralEntity::new_string(key), value));
    }
    (has_effect, result)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
    self.consume_self(analyzer);
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let definite = key_literals.len() == 1;
      let mut deleted = self.rest.borrow().values.len() > 0;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let mut string_keyed = self.string_keyed.borrow_mut();
            if definite {
              deleted |= string_keyed.remove(key).is_some();
            } else if let Some(property) = string_keyed.get_mut(key) {
              property.definite = false;
              deleted = true;
            }
          }
          LiteralEntity::Symbol(_) => todo!(),
          _ => unreachable!(),
        }
      }
      deleted
    } else {
      let mut string_keyed = self.string_keyed.borrow_mut();
      for property in string_keyed.values_mut() {
        property.definite = false;
      }
      true
    }
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    self.consume_as_unknown(analyzer);
    this.consume_as_unknown(analyzer);
    args.consume_as_unknown(analyzer);
    (true, UnknownEntity::new_unknown())
  }

  fn r#await(&self, rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    self.consume_as_unknown(analyzer);
    (true, UnknownEntity::new_unknown())
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

  fn get_to_array(&self, rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
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
  pub fn init_property(
    &self,
    kind: PropertyKind,
    key: Entity<'a>,
    value: Entity<'a>,
    definite: bool,
  ) {
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let definite = definite && key_literals.len() == 1;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let mut string_keyed = self.string_keyed.borrow_mut();
            let existing = string_keyed.get_mut(key);
            let reused_property = definite
              .then(|| {
                existing.and_then(|existing| {
                  for property in existing.values.iter() {
                    match property {
                      ObjectPropertyValue::Property(getter, setter) => {
                        return Some((getter.clone(), setter.clone()));
                      }
                      _ => {}
                    }
                  }
                  None
                })
              })
              .flatten();
            let property_val = match kind {
              PropertyKind::Init => ObjectPropertyValue::Field(value.clone()),
              PropertyKind::Get => ObjectPropertyValue::Property(
                Some(value.clone()),
                reused_property.and_then(|(_, setter)| setter),
              ),
              PropertyKind::Set => ObjectPropertyValue::Property(
                reused_property.and_then(|(getter, _)| getter),
                Some(value.clone()),
              ),
            };
            let existing = string_keyed.get_mut(key);
            if definite || existing.is_none() {
              let property = ObjectProperty { definite, values: vec![property_val] };
              string_keyed.insert(key, property);
            } else {
              existing.unwrap().values.push(property_val);
            }
          }
          LiteralEntity::Symbol(key) => todo!(),
          _ => unreachable!(),
        }
      }
    } else {
      self
        .unknown_keyed
        .borrow_mut()
        .values
        .push(ObjectPropertyValue::Field(EntryEntity::new(value, key)));
    }
  }

  pub fn init_spread(&self, analyzer: &mut Analyzer<'a>, argument: Entity<'a>) -> bool {
    let (has_effect, properties) = argument.enumerate_properties(analyzer);
    for (definite, key, value) in properties {
      self.init_property(PropertyKind::Init, key.clone(), value, definite);
    }
    has_effect
  }

  fn get_this(&self) -> Entity<'a> {
    UnknownEntity::new_unknown() // TODO: handle `this`
  }

  fn apply_unknown_to_possible_setters(&self, analyzer: &mut Analyzer<'a>) -> bool {
    fn apply_unknown_to_vec<'a>(
      analyzer: &mut Analyzer<'a>,
      property: &ObjectProperty<'a>,
      this: &Entity<'a>,
    ) -> bool {
      let mut has_effect = false;
      for property in &property.values {
        if let ObjectPropertyValue::Property(_, Some(setter)) = property {
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

  fn is_assignment_indeterminate(&self, analyzer: &Analyzer<'a>) -> bool {
    let mut var_scope_id = analyzer.scope_context.variable_scopes.first().unwrap().id;
    for (i, scope) in analyzer.scope_context.variable_scopes.iter().enumerate() {
      let scope_id = scope.id;
      if self.scope_path.get(i).is_some_and(|id| *id == scope_id) {
        var_scope_id = scope_id;
      } else {
        break;
      }
    }
    let target = analyzer.get_variable_scope_by_id(var_scope_id).cf_scope_id;
    analyzer.is_relative_indeterminate(target)
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_empty_object(&self) -> ObjectEntity<'a> {
    ObjectEntity {
      scope_path: self.variable_scope_path(),
      string_keyed: RefCell::new(FxHashMap::default()),
      unknown_keyed: RefCell::new(ObjectProperty::default()),
      rest: RefCell::new(ObjectProperty::default()),
    }
  }
}
