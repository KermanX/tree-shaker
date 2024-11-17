use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  builtins::Prototype,
  consumable::{box_consumable, Consumable, ConsumableCollector, ConsumableNode, ConsumableTrait},
  use_consumed_flag,
};
use oxc::{
  ast::ast::PropertyKind,
  semantic::{ScopeId, SymbolId},
};
use rustc_hash::FxHashMap;
use std::{
  cell::{Cell, RefCell},
  fmt, mem,
};

pub struct ObjectEntity<'a> {
  pub consumable: bool,
  consumed: Cell<bool>,
  deps: RefCell<ConsumableCollector<'a>>,
  cf_scope: ScopeId,
  object_id: SymbolId,
  pub string_keyed: RefCell<FxHashMap<&'a str, ObjectProperty<'a>>>,
  pub unknown_keyed: RefCell<ObjectProperty<'a>>,
  // TODO: symbol_keyed
  pub rest: RefCell<ObjectProperty<'a>>,
  pub prototype: &'a Prototype<'a>,
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
  /// (value, readonly)
  Field(Entity<'a>, Option<bool>),
  /// (Getter, Setter)
  Property(Option<Entity<'a>>, Option<Entity<'a>>),
}

impl<'a> ObjectPropertyValue<'a> {
  pub fn get_value(
    &self,
    analyzer: &mut Analyzer<'a>,
    suspended_getters: &mut Vec<Entity<'a>>,
  ) -> Option<Entity<'a>> {
    match self {
      ObjectPropertyValue::Field(value, _) => Some(value.clone()),
      ObjectPropertyValue::Property(Some(getter), _) => {
        suspended_getters.push(getter.clone());
        None
      }
      ObjectPropertyValue::Property(None, _) => Some(analyzer.factory.undefined),
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
    suspended_getters: &mut Vec<Entity<'a>>,
  ) -> Vec<Entity<'a>> {
    self
      .values
      .iter()
      .filter_map(|property| property.get_value(analyzer, suspended_getters))
      .collect()
  }
}

impl<'a> EntityTrait<'a> for ObjectEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    if !self.consumable {
      return;
    }

    use_consumed_flag!(self);

    analyzer.mark_object_consumed(self.cf_scope, self.object_id);

    self.deps.take().consume_all(analyzer);

    fn consume_property<'a>(property: &ObjectProperty<'a>, analyzer: &mut Analyzer<'a>) {
      for value in &property.values {
        match value {
          ObjectPropertyValue::Field(value, _) => value.consume(analyzer),
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

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    if self.consumed.get() {
      return consumed_object::unknown_mutate(analyzer, dep);
    }

    let (has_exhaustive, _, exec_deps) = analyzer.pre_must_mutate(self.cf_scope, self.object_id);

    if has_exhaustive {
      self.consume(analyzer);
      return consumed_object::unknown_mutate(analyzer, dep);
    }

    self.deps.borrow_mut().push(box_consumable((exec_deps, dep)));
  }

  fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_property(rc, analyzer, dep, key);
    }

    // FIXME: this is inaccurate - the read properties may be all getter/setters
    analyzer.mark_object_property_exhaustive_read(self.cf_scope, self.object_id);

    analyzer.push_indeterminate_cf_scope();

    let key = key.get_to_property_key(analyzer);
    let value = if let Some(key_literals) = key.get_to_literals(analyzer) {
      let mut suspended_getters = vec![];
      let mut values = self.unknown_keyed.borrow().get_value(analyzer, &mut suspended_getters);
      let mut rest_added = false;
      let mut undefined_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let lookup_rest = if let Some(property) = self.string_keyed.borrow().get(key) {
              values.extend(property.get_value(analyzer, &mut suspended_getters));
              !property.definite
            } else {
              true
            };
            let add_undefined = if lookup_rest {
              if let Some(from_prototype) = self.prototype.get_string_keyed(key) {
                values.push(from_prototype.clone());
              }
              if !rest_added {
                rest_added = true;
                let rest = self.rest.borrow();
                values.extend(rest.get_value(analyzer, &mut suspended_getters));
                true
              } else {
                false
              }
            } else {
              false
            };
            if add_undefined && !undefined_added {
              undefined_added = true;
              values.push(analyzer.factory.undefined);
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!(),
        }
      }
      let getter_args = analyzer.factory.arguments(vec![]);
      values.extend(
        suspended_getters.into_iter().map(|f| f.call(analyzer, dep.cloned(), rc, getter_args)),
      );
      analyzer.factory.computed(
        analyzer.factory.union(values),
        (dep, key.clone(), self.deps.borrow_mut().collect()),
      )
    } else {
      if analyzer.is_inside_pure() {
        analyzer.factory.computed_unknown((rc, dep, key))
      } else {
        // TODO: like set_property, call getters and collect all possible values
        // FIXME: if analyzer.config.unknown_property_read_side_effects {
        self.consume(analyzer);
        // }
        consumed_object::get_property(rc, analyzer, dep.cloned(), key)
      }
    };

    analyzer.pop_cf_scope();

    value
  }

  fn set_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    if self.consumed.get() {
      return consumed_object::set_property(analyzer, dep, key, value);
    }

    let target_depth = analyzer.find_first_different_cf_scope(self.cf_scope);
    let (has_exhaustive, indeterminate, exec_deps) = analyzer.pre_possible_mutate(target_depth);
    let dep_cloned = dep.cloned();

    analyzer.push_indeterminate_cf_scope();

    let key = key.get_to_property_key(analyzer);
    let value = analyzer.factory.computed(value, key);
    let this = rc;

    let mut may_write = false;

    if let Some(key_literals) = key.get_to_literals(analyzer) {
      let indeterminate = indeterminate
        || self.unknown_keyed.borrow().values.len() > 0
        || self.rest.borrow().values.len() > 0;
      let definite = !indeterminate && key_literals.len() == 1;
      let mut rest_and_unknown_setter_called = false;
      let mut suspended_setters = vec![];
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let mut string_keyed = self.string_keyed.borrow_mut();
            if let Some(property) = string_keyed.get_mut(key) {
              let has_writable_field = if definite {
                let prev_len = property.values.len();
                property.values = property
                  .values
                  .iter()
                  .filter(|v| {
                    matches!(
                      v,
                      ObjectPropertyValue::Property(_, _)
                        | ObjectPropertyValue::Field(_, Some(true))
                    )
                  })
                  .cloned()
                  .collect::<Vec<_>>();
                prev_len != property.values.len()
              } else {
                true
              };
              for property_val in
                property.values.iter().chain(self.unknown_keyed.borrow().values.iter())
              {
                if let ObjectPropertyValue::Property(_, Some(setter)) = property_val {
                  suspended_setters.push(*setter);
                }
              }
              if indeterminate || has_writable_field || self.unknown_keyed.borrow().values.len() > 0
              {
                may_write = true;
                property.values.push(ObjectPropertyValue::Field(value.clone(), Some(false)));
              }
            } else {
              may_write = true;

              // Call setters in rest and unknown_keyed
              if !rest_and_unknown_setter_called {
                rest_and_unknown_setter_called = true;
                let rest = self.rest.borrow_mut();
                for property in rest.values.iter().chain(self.unknown_keyed.borrow().values.iter())
                {
                  if let ObjectPropertyValue::Property(_, Some(setter)) = property {
                    setter.call(
                      analyzer,
                      dep.cloned(),
                      this,
                      analyzer.factory.arguments(vec![(false, value.clone())]),
                    );
                  }
                }
              }

              let property = ObjectProperty {
                definite,
                values: vec![ObjectPropertyValue::Field(value.clone(), Some(false))],
              };
              string_keyed.insert(key, property);
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!(),
        }
      }

      for setter in suspended_setters {
        setter.call(
          analyzer,
          dep.cloned(),
          this,
          analyzer.factory.arguments(vec![(false, value.clone())]),
        );
      }
    } else {
      may_write = true;
      self
        .unknown_keyed
        .borrow_mut()
        .values
        .push(ObjectPropertyValue::Field(analyzer.factory.computed(value, key), None));
      self.apply_unknown_to_possible_setters(analyzer, dep);
    };

    analyzer.pop_cf_scope();

    if may_write {
      self.add_assignment_dep(exec_deps, dep_cloned);

      if has_exhaustive {
        self.consume(analyzer);
        analyzer.mark_object_property_exhaustive_write(target_depth, self.object_id);
      }
    }
  }

  fn enumerate_properties(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    if self.consumed.get() {
      return consumed_object::enumerate_properties(rc, analyzer, dep);
    }

    // FIXME: this is inaccurate - the read properties may be all getter/setters
    analyzer.mark_object_property_exhaustive_read(self.cf_scope, self.object_id);

    // unknown_keyed = unknown_keyed + rest
    let mut suspended_getters = vec![];
    let mut unknown_keyed = self.unknown_keyed.borrow().get_value(analyzer, &mut suspended_getters);
    unknown_keyed.extend(self.rest.borrow().get_value(analyzer, &mut suspended_getters));
    let getter_args = analyzer.factory.arguments(vec![]);
    unknown_keyed.extend(
      suspended_getters.into_iter().map(|f| f.call(analyzer, dep.cloned(), rc, getter_args)),
    );
    let mut result = Vec::new();
    if unknown_keyed.len() > 0 {
      result.push((
        false,
        analyzer.factory.unknown_primitive,
        analyzer.factory.union(unknown_keyed),
      ));
    }

    let string_keyed = self.string_keyed.borrow();
    let keys = string_keyed.keys().cloned().collect::<Vec<_>>();
    mem::drop(string_keyed);
    for key in keys {
      let string_keyed = self.string_keyed.borrow();
      let properties = string_keyed.get(&key).unwrap();
      let definite = properties.definite;
      let mut suspended_getters = vec![];
      let mut values = properties.get_value(analyzer, &mut suspended_getters);
      mem::drop(string_keyed);
      values.extend(
        suspended_getters.into_iter().map(|f| f.call(analyzer, dep.cloned(), rc, getter_args)),
      );
      result.push((definite, analyzer.factory.string(key), analyzer.factory.union(values)));
    }

    (result, box_consumable((self.deps.borrow_mut().collect(), dep.cloned())))
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    if self.consumed.get() {
      return consumed_object::delete_property(analyzer, dep, key);
    }

    let target_depth = analyzer.find_first_different_cf_scope(self.cf_scope);
    let (has_exhaustive, indeterminate, exec_deps) = analyzer.pre_possible_mutate(target_depth);

    let key = key.get_to_property_key(analyzer);
    let may_delete = if let Some(key_literals) = key.get_to_literals(analyzer) {
      let definite = key_literals.len() == 1;
      let mut may_delete = self.unknown_keyed.borrow().values.len() > 0;
      let has_rest = !self.rest.borrow().values.is_empty();
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            let mut string_keyed = self.string_keyed.borrow_mut();
            if definite && !indeterminate {
              let removed = string_keyed.remove(key);
              may_delete |= removed.is_some();
              if !has_rest && removed.map_or(true, |property| !property.definite) {
                may_delete = true;
              }
            } else if let Some(property) = string_keyed.get_mut(key) {
              property.definite = false;
              may_delete = true;
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!(),
        }
      }
      may_delete
    } else {
      let mut string_keyed = self.string_keyed.borrow_mut();
      for property in string_keyed.values_mut() {
        property.definite = false;
      }
      true
    };

    if may_delete {
      self.add_assignment_dep(exec_deps, (dep, key));
      if has_exhaustive {
        self.consume(analyzer);
        analyzer.mark_object_property_exhaustive_write(target_depth, self.object_id);
      }
    }
  }

  fn call(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    consumed_object::call(rc, analyzer, dep, this, args)
  }

  fn construct(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    consumed_object::construct(rc, analyzer, dep, args)
  }

  fn jsx(&self, rc: Entity<'a>, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    consumed_object::jsx(rc, analyzer, props)
  }

  fn r#await(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    self.consume(analyzer);
    consumed_object::r#await(analyzer, dep)
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_destructable(&self, _rc: Entity<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    dep
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.string("object")
  }

  fn get_to_string(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    // FIXME: Special methods
    if self.consumed.get() {
      return consumed_object::get_to_string(analyzer);
    }
    analyzer.factory.computed_unknown_string(rc)
  }

  fn get_to_numeric(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    // FIXME: Special methods
    if self.consumed.get() {
      return consumed_object::get_to_numeric(analyzer);
    }
    analyzer.factory.computed_unknown(rc)
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.boolean(true)
  }

  fn get_to_property_key(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.get_to_string(rc, analyzer)
  }

  fn get_to_jsx_child(&self, rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    rc
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
  pub fn new_builtin(object_id: SymbolId, prototype: &'a Prototype<'a>, consumable: bool) -> Self {
    ObjectEntity {
      consumable,
      consumed: Cell::new(false),
      deps: Default::default(),
      cf_scope: ScopeId::new(0),
      object_id,
      string_keyed: Default::default(),
      unknown_keyed: Default::default(),
      rest: Default::default(),
      prototype,
    }
  }

  pub fn init_property(
    &self,
    analyzer: &mut Analyzer<'a>,
    kind: PropertyKind,
    key: Entity<'a>,
    value: Entity<'a>,
    definite: bool,
  ) {
    let value = analyzer.factory.computed(value, key);
    if let Some(key_literals) = key.get_to_literals(analyzer) {
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
              PropertyKind::Init => ObjectPropertyValue::Field(value.clone(), Some(false)),
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
      let property_val = match kind {
        PropertyKind::Init => ObjectPropertyValue::Field(value.clone(), Some(false)),
        PropertyKind::Get => ObjectPropertyValue::Property(Some(value.clone()), None),
        PropertyKind::Set => ObjectPropertyValue::Property(None, Some(value.clone())),
      };
      self.unknown_keyed.borrow_mut().values.push(property_val);
    }
  }

  pub fn init_spread(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    argument: Entity<'a>,
  ) {
    let (properties, deps) = argument.enumerate_properties(analyzer, dep);
    self.deps.borrow_mut().push(deps);
    for (definite, key, value) in properties {
      self.init_property(analyzer, PropertyKind::Init, key, value, definite);
    }
  }

  fn apply_unknown_to_possible_setters(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    fn apply_unknown_to_vec<'a>(
      analyzer: &mut Analyzer<'a>,
      dep: Consumable<'a>,
      property: &ObjectProperty<'a>,
    ) {
      for property in &property.values {
        if let ObjectPropertyValue::Property(_, Some(setter)) = property {
          setter.call(
            analyzer,
            dep.cloned(),
            analyzer.factory.unknown(),
            analyzer.factory.arguments(vec![(false, analyzer.factory.unknown())]),
          );
        }
      }
    }

    for property in self.string_keyed.borrow().values() {
      apply_unknown_to_vec(analyzer, dep.cloned(), property);
    }
    apply_unknown_to_vec(analyzer, dep.cloned(), &mut self.unknown_keyed.borrow());
    apply_unknown_to_vec(analyzer, dep.cloned(), &self.rest.borrow());
  }

  fn add_assignment_dep<T: ConsumableTrait<'a> + 'a>(
    &self,
    exec_deps: ConsumableNode<'a, T>,
    dep: impl ConsumableTrait<'a> + 'a,
  ) {
    let mut deps = self.deps.borrow_mut();
    deps.push(box_consumable((exec_deps, dep)));
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_empty_object(&mut self, prototype: &'a Prototype<'a>) -> ObjectEntity<'a> {
    ObjectEntity {
      consumable: true,
      consumed: Cell::new(false),
      deps: Default::default(),
      cf_scope: self.scope_context.cf.current_id(),
      object_id: self.scope_context.alloc_object_id(),
      string_keyed: RefCell::new(FxHashMap::default()),
      unknown_keyed: RefCell::new(ObjectProperty::default()),
      rest: RefCell::new(ObjectProperty::default()),
      prototype,
    }
  }
}
