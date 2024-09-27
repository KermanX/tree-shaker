use super::{
  consumed_object, ArgumentsEntity, Consumable, Entity, EntityTrait, EntryEntity, ForwardedEntity,
  InteractionKind, LiteralEntity, TypeofResult, UnionEntity, UnknownEntity,
};
use crate::{
  analyzer::Analyzer,
  scope::{cf_scope::CfScopes, variable_scope::VariableScopes},
  use_consumed_flag,
};
use oxc::ast::ast::PropertyKind;
use rustc_hash::FxHashMap;
use std::{
  cell::{Cell, RefCell},
  fmt, mem,
};

#[derive(Clone, Default)]
pub struct ObjectEntity<'a> {
  consumed: Cell<bool>,
  deps: RefCell<Vec<Consumable<'a>>>,
  cf_scopes: CfScopes<'a>,
  variable_scopes: VariableScopes<'a>,
  pub string_keyed: RefCell<FxHashMap<&'a str, ObjectProperty<'a>>>,
  pub unknown_keyed: RefCell<ObjectProperty<'a>>,
  // TODO: symbol_keyed
  pub rest: RefCell<ObjectProperty<'a>>,
}

impl<'a> fmt::Debug for ObjectEntity<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ObjectEntity")
      .field("consumed", &self.consumed.get())
      .field("deps", &self.deps)
      .field("string_keyed", &self.string_keyed)
      .field("unknown_keyed", &self.unknown_keyed)
      .field("rest", &self.rest)
      .finish()
  }
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
    dep: Consumable<'a>,
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
    dep: Consumable<'a>,
    this: &Entity<'a>,
  ) -> Vec<Entity<'a>> {
    self.values.iter().map(|property| property.get_value(analyzer, dep.clone(), this)).collect()
  }
}

impl<'a> EntityTrait<'a> for ObjectEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    analyzer.consume(mem::take(&mut *self.deps.borrow_mut()));

    fn consume_property<'a>(property: &ObjectProperty<'a>, analyzer: &mut Analyzer<'a>) {
      for value in &property.values {
        match value {
          ObjectPropertyValue::Field(value) => value.consume(analyzer),
          ObjectPropertyValue::Property(getter, setter) => {
            getter.as_ref().map(|f| f.consume(analyzer));
            setter.as_ref().map(|f| f.consume(analyzer));
          }
        }
      }
    }

    for property in self.string_keyed.borrow().values() {
      consume_property(property, analyzer);
    }
    consume_property(&self.rest.borrow(), analyzer);
    consume_property(&self.unknown_keyed.borrow(), analyzer);
  }

  fn interact(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, kind: InteractionKind) {
    self.consume(analyzer);
    consumed_object::interact(analyzer, dep, kind)
  }

  fn get_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_property(analyzer, dep, key);
    }
    analyzer.exec_indeterminately(move |analyzer| {
      let this = rc.clone();
      if let Some(key_literals) = key.get_to_property_key().get_to_literals() {
        let mut values = self.unknown_keyed.borrow().get_value(analyzer, dep.clone(), &this);
        let mut rest_added = false;
        let mut undefined_added = false;
        for key_literal in key_literals {
          match key_literal {
            LiteralEntity::String(key) => {
              let string_keyed = self.string_keyed.borrow();
              let lookup_rest = if let Some(property) = string_keyed.get(key) {
                values.extend(property.get_value(analyzer, dep.clone(), &this));
                !property.definite
              } else {
                true
              };
              let add_undefined = if lookup_rest {
                if let Some(from_prototype) = analyzer.builtins.prototypes.object.get(key) {
                  values.push(from_prototype.clone());
                }
                if !rest_added {
                  rest_added = true;
                  let rest = self.rest.borrow();
                  values.extend(rest.get_value(analyzer, dep.clone(), &this));
                  true
                } else {
                  false
                }
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
        ForwardedEntity::new(
          EntryEntity::new(UnionEntity::new(values), (dep, key.clone())),
          self.deps.borrow().clone(),
        )
      } else {
        // TODO: like set_property, call getters and collect all possible values
        self.consume(analyzer);
        consumed_object::get_property(analyzer, dep, key)
      }
    })
  }

  fn set_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    if self.consumed.get() {
      return consumed_object::set_property(analyzer, dep, key, value);
    }
    let indeterminate = analyzer.is_assignment_indeterminate(&self.cf_scopes);
    analyzer.exec_indeterminately(move |analyzer| {
      self.add_assignment_dep(analyzer, dep.clone());
      let this = rc.clone();
      let key = key.get_to_property_key();
      if let Some(key_literals) = key.get_to_literals() {
        let indeterminate = indeterminate
          || self.unknown_keyed.borrow().values.len() > 0
          || self.rest.borrow().values.len() > 0;
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
                  for property in
                    rest.values.iter().chain(self.unknown_keyed.borrow().values.iter())
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
    })
  }

  fn enumerate_properties(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    if self.consumed.get() {
      return consumed_object::enumerate_properties(analyzer, dep);
    }
    let mut deps = self.deps.borrow().clone();
    deps.push(dep.clone());
    let self_dep = Consumable::new(deps);

    let this = rc.clone();
    // unknown_keyed = unknown_keyed + rest
    let mut unknown_keyed = self.unknown_keyed.borrow().get_value(analyzer, dep.clone(), &this);
    unknown_keyed.extend(self.rest.borrow().get_value(analyzer, dep.clone(), &this));
    let mut result = Vec::new();
    if unknown_keyed.len() > 0 {
      result.push((
        false,
        UnknownEntity::new_unknown(),
        ForwardedEntity::new(UnionEntity::new(unknown_keyed), self_dep.clone()),
      ));
    }
    for (key, properties) in self.string_keyed.borrow().iter() {
      let values = properties.get_value(analyzer, dep.clone(), &this);
      result.push((
        properties.definite,
        LiteralEntity::new_string(key),
        ForwardedEntity::new(UnionEntity::new(values), self_dep.clone()),
      ));
    }
    result
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: &Entity<'a>) {
    if self.consumed.get() {
      return consumed_object::delete_property(analyzer, dep, key);
    }
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
      if deleted {
        self.add_assignment_dep(analyzer, dep);
      }
    } else {
      let mut string_keyed = self.string_keyed.borrow_mut();
      for property in string_keyed.values_mut() {
        property.definite = false;
      }
      self.add_assignment_dep(analyzer, dep);
    }
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.consume(analyzer);
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    self.consume(analyzer);
    consumed_object::r#await(analyzer, dep)
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("object")
  }

  fn get_to_string(&self, rc: &Entity<'a>) -> Entity<'a> {
    // FIXME: Special methods
    if self.consumed.get() {
      return consumed_object::get_to_string();
    }
    UnknownEntity::new_computed_string(rc.clone())
  }

  fn get_to_numeric(&self, rc: &Entity<'a>) -> Entity<'a> {
    // FIXME: Special methods
    if self.consumed.get() {
      return consumed_object::get_to_numeric();
    }
    UnknownEntity::new_computed_unknown(rc.clone())
  }

  fn get_to_property_key(&self, rc: &Entity<'a>) -> Entity<'a> {
    self.get_to_string(rc)
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
      let value = EntryEntity::new(value, key);
      let property_val = match kind {
        PropertyKind::Init => ObjectPropertyValue::Field(value.clone()),
        PropertyKind::Get => ObjectPropertyValue::Property(Some(value.clone()), None),
        PropertyKind::Set => ObjectPropertyValue::Property(None, Some(value.clone())),
      };
      self.unknown_keyed.borrow_mut().values.push(property_val);
    }
  }

  pub fn init_spread(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: impl Into<Consumable<'a>>,
    argument: Entity<'a>,
  ) {
    let properties = argument.enumerate_properties(analyzer, dep);
    for (definite, key, value) in properties {
      self.init_property(PropertyKind::Init, key.clone(), value, definite);
    }
  }

  fn apply_unknown_to_possible_setters(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    fn apply_unknown_to_vec<'a>(
      analyzer: &mut Analyzer<'a>,
      dep: Consumable<'a>,
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

  fn add_assignment_dep(&self, analyzer: &Analyzer<'a>, dep: Consumable<'a>) {
    let target_variable_scope = analyzer.find_first_different_variable_scope(&self.variable_scopes);
    self.deps.borrow_mut().push(analyzer.get_assignment_deps(target_variable_scope, dep));
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_empty_object(&self) -> ObjectEntity<'a> {
    ObjectEntity {
      consumed: Cell::new(false),
      deps: RefCell::new(Vec::new()),
      cf_scopes: self.scope_context.cf_scopes.clone(),
      variable_scopes: self.scope_context.variable_scopes.clone(),
      string_keyed: RefCell::new(FxHashMap::default()),
      unknown_keyed: RefCell::new(ObjectProperty::default()),
      rest: RefCell::new(ObjectProperty::default()),
    }
  }
}
