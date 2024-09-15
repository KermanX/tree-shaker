use super::{
  arguments::ArgumentsEntity,
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
use oxc::ast::ast::PropertyKind;
use rustc_hash::FxHashMap;
use std::cell::{Cell, RefCell};

#[derive(Debug, Clone)]
pub struct ObjectEntity<'a> {
  pub consumed: Cell<bool>,
  pub cf_scopes: CfScopes<'a>,
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
  pub fn get_value(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
  ) -> Entity<'a> {
    match self {
      ObjectPropertyValue::Field(value) => value.clone(),
      ObjectPropertyValue::Property(Some(getter), _) => {
        getter.call(analyzer, dep, this, &ArgumentsEntity::new(vec![]))
      }
      _ => LiteralEntity::new_undefined(),
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
    dep: EntityDep,
    this: &Entity<'a>,
  ) -> Vec<Entity<'a>> {
    self.values.iter().map(|property| property.get_value(analyzer, dep.clone(), this)).collect()
  }
}

impl<'a> EntityTrait<'a> for ObjectEntity<'a> {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);
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

  fn get_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_property(analyzer, dep, key);
    }
    let this = rc.clone();
    if let Some(key_literals) = key.get_to_property_key().get_to_literals() {
      let mut values = self.unknown_keyed.borrow().get_value(analyzer, dep.clone(), &this);
      let mut rest_added = false;
      let mut undefined_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let string_keyed = self.string_keyed.borrow();
            let add_undefined = if let Some(property) = string_keyed.get(key) {
              values.extend(property.get_value(analyzer, dep.clone(), &this));
              !property.definite
            } else if !rest_added {
              rest_added = true;
              let rest = self.rest.borrow();
              values.extend(rest.get_value(analyzer, dep.clone(), &this));
              true
            } else {
              false
            };
            if add_undefined && !undefined_added {
              undefined_added = true;
              values.push(LiteralEntity::new_undefined());
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!(),
        }
      }
      EntryEntity::new(UnionEntity::new(values), key.clone())
    } else {
      // TODO: like set_property, call getters and collect all possible values
      self.consume_as_unknown(analyzer);
      consumed_object::get_property(analyzer, dep, key)
    }
  }

  fn set_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    if self.consumed.get() {
      return consumed_object::set_property(analyzer, dep, key, value);
    }
    let this = rc.clone();
    let indeterminate = analyzer.is_assignment_indeterminate(&self.cf_scopes);
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let indeterminate = indeterminate || self.unknown_keyed.borrow().values.len() > 0;
      let definite = !indeterminate && key_literals.len() == 1;
      let mut rest_set = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let mut string_keyed = self.string_keyed.borrow_mut();
            if let Some(property) = string_keyed.get_mut(key) {
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
                  setter.call(
                    analyzer,
                    dep.clone(),
                    &this,
                    &ArgumentsEntity::new(vec![(false, value.clone())]),
                  );
                }
              }
              if indeterminate || !property.definite || property.values.is_empty() {
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
                    setter.call(
                      analyzer,
                      dep.clone(),
                      &this,
                      &ArgumentsEntity::new(vec![(false, value.clone())]),
                    );
                  }
                }
              }

              let property = ObjectProperty {
                definite,
                values: vec![ObjectPropertyValue::Field(value.clone())],
              };
              string_keyed.insert(key, property);
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!(),
        }
      }
    } else {
      self
        .unknown_keyed
        .borrow_mut()
        .values
        .push(ObjectPropertyValue::Field(EntryEntity::new(value, key)));
      self.apply_unknown_to_possible_setters(analyzer, dep)
    }
  }

  fn enumerate_properties(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    if self.consumed.get() {
      return consumed_object::enumerate_properties(analyzer, dep);
    }
    let this = rc.clone();
    // unknown_keyed = unknown_keyed + rest
    let mut unknown_keyed = self.unknown_keyed.borrow().get_value(analyzer, dep.clone(), &this);
    unknown_keyed.extend(self.rest.borrow().get_value(analyzer, dep.clone(), &this));
    let mut result = Vec::new();
    if unknown_keyed.len() > 0 {
      result.push((false, UnknownEntity::new_unknown(), UnionEntity::new(unknown_keyed)));
    }
    for (key, properties) in self.string_keyed.borrow().iter() {
      let values = properties.get_value(analyzer, dep.clone(), &this);
      result.push((properties.definite, LiteralEntity::new_string(key), UnionEntity::new(values)));
    }
    result
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
    if self.consumed.get() {
      return consumed_object::delete_property(analyzer, key);
    }
    self.consume_self(analyzer);
    let indeterminate = analyzer.is_assignment_indeterminate(&self.cf_scopes);
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let definite = key_literals.len() == 1;
      let mut deleted = self.rest.borrow().values.len() > 0;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let mut string_keyed = self.string_keyed.borrow_mut();
            if definite && !indeterminate {
              deleted |= string_keyed.remove(key).is_some();
            } else if let Some(property) = string_keyed.get_mut(key) {
              property.definite = false;
              deleted = true;
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
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
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.consume_as_unknown(analyzer);
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    self.consume_as_unknown(analyzer);
    consumed_object::r#await(analyzer)
  }

  fn iterate(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Option<Entity<'a>>) {
    self.consume_as_unknown(analyzer);
    consumed_object::iterate(analyzer)
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

  fn get_to_array(&self, rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    if self.consumed.get() {
      return consumed_object::get_to_array(length);
    }
    UnknownEntity::new_unknown_to_array_result(length, vec![rc.clone()])
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
          LiteralEntity::Symbol(key, _) => todo!(),
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

  pub fn init_spread(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<EntityDep>,
    argument: Entity<'a>,
  ) {
    let properties = argument.enumerate_properties(analyzer, dep);
    for (definite, key, value) in properties {
      self.init_property(PropertyKind::Init, key.clone(), value, definite);
    }
  }

  fn apply_unknown_to_possible_setters(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep) {
    fn apply_unknown_to_vec<'a>(
      analyzer: &mut Analyzer<'a>,
      dep: EntityDep,
      property: &ObjectProperty<'a>,
      this: &Entity<'a>,
    ) {
      for property in &property.values {
        if let ObjectPropertyValue::Property(_, Some(setter)) = property {
          setter.call(
            analyzer,
            dep.clone(),
            this,
            &ArgumentsEntity::new(vec![(false, UnknownEntity::new_unknown())]),
          );
        }
      }
    }

    for property in self.string_keyed.borrow().values() {
      apply_unknown_to_vec(analyzer, dep.clone(), property, &UnknownEntity::new_unknown());
    }
    apply_unknown_to_vec(
      analyzer,
      dep.clone(),
      &mut self.unknown_keyed.borrow(),
      &UnknownEntity::new_unknown(),
    );
    apply_unknown_to_vec(
      analyzer,
      dep.clone(),
      &mut self.rest.borrow(),
      &UnknownEntity::new_unknown(),
    );
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_empty_object(&self) -> ObjectEntity<'a> {
    ObjectEntity {
      consumed: Cell::new(false),
      cf_scopes: self.scope_context.cf_scopes.clone(),
      string_keyed: RefCell::new(FxHashMap::default()),
      unknown_keyed: RefCell::new(ObjectProperty::default()),
      rest: RefCell::new(ObjectProperty::default()),
    }
  }
}
