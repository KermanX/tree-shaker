use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  builtins::Prototype,
  consumable::{box_consumable, Consumable, ConsumableCollector, ConsumableNode, ConsumableTrait},
  scope::CfScopeKind,
  use_consumed_flag,
};
use oxc::{
  ast::ast::PropertyKind,
  semantic::{ScopeId, SymbolId},
};
use rustc_hash::FxHashMap;
use std::{
  cell::{Cell, RefCell},
  mem,
};

#[derive(Debug)]
pub struct ObjectEntity<'a> {
  /// A built-in object is usually non-consumable
  pub consumable: bool,
  consumed: Cell<bool>,
  // deps: RefCell<ConsumableCollector<'a>>,
  /// Where the object is created
  cf_scope: ScopeId,
  pub object_id: SymbolId,
  pub string_keyed: RefCell<FxHashMap<&'a str, ObjectProperty<'a>>>,
  pub unknown_keyed: RefCell<ObjectProperty<'a>>,
  // TODO: symbol_keyed
  pub rest: RefCell<Option<ObjectProperty<'a>>>,
  pub prototype: &'a Prototype<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectPropertyValue<'a> {
  /// (value, readonly)
  Field(Entity<'a>, bool),
  /// (getter, setter)
  Property(Option<Entity<'a>>, Option<Entity<'a>>),
}

#[derive(Debug)]
pub struct ObjectProperty<'a> {
  pub definite: bool,
  pub possible_values: Vec<ObjectPropertyValue<'a>>,
  pub non_existent: ConsumableCollector<'a>,
}

impl<'a> Default for ObjectProperty<'a> {
  fn default() -> Self {
    Self { definite: true, possible_values: vec![], non_existent: ConsumableCollector::default() }
  }
}

impl<'a> ObjectProperty<'a> {
  pub fn get(
    &mut self,
    analyzer: &Analyzer<'a>,
    values: &mut Vec<Entity<'a>>,
    getters: &mut Vec<Entity<'a>>,
    non_existent: &mut Vec<ConsumableNode<'a>>,
  ) {
    for possible_value in &self.possible_values {
      match possible_value {
        ObjectPropertyValue::Field(value, _) => values.push(*value),
        ObjectPropertyValue::Property(Some(getter), _) => getters.push(*getter),
        ObjectPropertyValue::Property(None, _) => values.push(analyzer.factory.undefined),
      }
    }

    if let Some(dep) = self.non_existent.try_collect() {
      non_existent.push(dep);
    } else if !self.definite && non_existent.is_empty() {
      non_existent.push(ConsumableNode::new_box(()));
    }
  }

  pub fn set(
    &mut self,
    indeterminate: bool,
    value: Entity<'a>,
    setters: &mut Vec<(bool, Option<ConsumableNode<'a>>, Entity<'a>)>,
  ) {
    let mut writable = false;
    let call_setter_indeterminately = indeterminate || self.possible_values.len() > 1;
    for possible_value in &self.possible_values {
      match *possible_value {
        ObjectPropertyValue::Field(_, false) => writable = true,
        ObjectPropertyValue::Property(_, Some(setter)) => {
          setters.push((call_setter_indeterminately, self.non_existent.try_collect(), setter))
        }
        _ => {}
      }
    }

    if !indeterminate {
      // Remove all writable fields
      self.possible_values = self
        .possible_values
        .iter()
        .filter(|possible_value| !matches!(possible_value, ObjectPropertyValue::Field(_, false)))
        .cloned()
        .collect();
      // This property must exist now
      self.non_existent.force_clear();
    }

    if writable {
      self.possible_values.push(ObjectPropertyValue::Field(value, false));
    }
  }

  pub fn delete(&mut self, indeterminate: bool, dep: Consumable<'a>) {
    self.definite = false;
    if !indeterminate {
      self.possible_values.clear();
      self.non_existent.force_clear();
    }
    self.non_existent.push(dep);
  }

  pub fn consume(self, analyzer: &mut Analyzer<'a>) {
    for possible_value in self.possible_values {
      match possible_value {
        ObjectPropertyValue::Field(value, _) => analyzer.consume(value),
        ObjectPropertyValue::Property(getter, setter) => {
          analyzer.consume(getter);
          analyzer.consume(setter);
        }
      }
    }

    self.non_existent.consume_all(analyzer);
  }
}

impl<'a> EntityTrait<'a> for ObjectEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    if !self.consumable {
      return;
    }

    use_consumed_flag!(self);

    // self.deps.take().consume_all(analyzer);

    for property in self.string_keyed.take().into_values() {
      property.consume(analyzer);
    }
    self.unknown_keyed.take().consume(analyzer);

    analyzer.mark_object_consumed(self.cf_scope, self.object_id);
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    // if self.consumed.get() {
    //   return consumed_object::unknown_mutate(analyzer, dep);
    // }

    // let (has_exhaustive, _, exec_deps) = analyzer.pre_must_mutate(self.cf_scope, self.object_id);

    // if has_exhaustive {
    //   self.consume(analyzer);
    //   return consumed_object::unknown_mutate(analyzer, dep);
    // }

    // self.deps.borrow_mut().push(box_consumable((exec_deps, dep)));
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

    analyzer.mark_object_property_exhaustive_read(self.cf_scope, self.object_id);

    let mut values = vec![];
    let mut getters = vec![];
    let mut non_existent = vec![];

    let mut check_rest = false;
    let mut may_add_undefined = false;
    let key = key.get_to_property_key(analyzer);
    if let Some(key_literals) = key.get_to_literals(analyzer) {
      let mut string_keyed = self.string_keyed.borrow_mut();
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(property) = string_keyed.get_mut(key) {
              property.get(analyzer, &mut values, &mut getters, &mut non_existent);
            } else {
              check_rest = true;
              if let Some(property) = self.prototype.get_string_keyed(key) {
                values.push(property);
              } else {
                may_add_undefined = true;
              }
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }

      check_rest |= non_existent.len() > 0;
      may_add_undefined |= non_existent.len() > 0;
    } else {
      for property in self.string_keyed.borrow_mut().values_mut() {
        property.get(analyzer, &mut values, &mut getters, &mut non_existent);
      }

      // TODO: prototype? Use a config IMO
      // Either:
      // - Skip prototype
      // - Return unknown and call all getters

      check_rest = true;
      may_add_undefined = true;
    }

    if check_rest {
      let mut rest = self.rest.borrow_mut();
      if let Some(rest) = &mut *rest {
        rest.get(analyzer, &mut values, &mut getters, &mut non_existent);
      } else if may_add_undefined {
        values.push(analyzer.factory.undefined);
      }
    }

    let indeterminate_getter = values.len() > 0 || getters.len() > 1 || non_existent.len() > 0;

    {
      let mut unknown_keyed = self.unknown_keyed.borrow_mut();
      unknown_keyed.get(analyzer, &mut values, &mut getters, &mut non_existent);
    }

    if getters.len() > 0 {
      if indeterminate_getter {
        analyzer.push_indeterminate_cf_scope();
      }
      for getter in getters {
        values.push(getter.call_as_getter(analyzer, box_consumable((dep.cloned(), key)), rc));
      }
      if indeterminate_getter {
        analyzer.pop_cf_scope();
      }
    }

    let dep = box_consumable(ConsumableNode::new((non_existent, dep, key)));
    analyzer
      .factory
      .computed(analyzer.factory.try_union(values).unwrap_or(analyzer.factory.undefined), dep)
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

    let (has_exhaustive, mut indeterminate, exec_deps) =
      analyzer.pre_mutate_object(self.cf_scope, self.object_id);

    if has_exhaustive {
      self.consume(analyzer);
      return consumed_object::set_property(analyzer, dep, key, value);
    }

    let key = key.get_to_property_key(analyzer);
    let value = analyzer.factory.computed(value, (exec_deps, dep.cloned(), key));

    let mut setters = vec![];

    {
      let unknown_keyed = self.unknown_keyed.borrow();
      for possible_value in &unknown_keyed.possible_values {
        if let ObjectPropertyValue::Property(_, setter) = possible_value {
          if let Some(setter) = setter {
            setters.push((true, None, setter.clone()));
          }
          indeterminate = true;
        }
      }
    }

    if let Some(key_literals) = key.get_to_literals(analyzer) {
      indeterminate |= key_literals.len() > 1;

      let mut string_keyed = self.string_keyed.borrow_mut();
      let mut rest = self.rest.borrow_mut();
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(property) = string_keyed.get_mut(key) {
              property.set(indeterminate, value, &mut setters);
            } else if let Some(rest) = &mut *rest {
              rest.set(true, value, &mut setters);
            } else {
              string_keyed.insert(
                key,
                ObjectProperty {
                  definite: !indeterminate,
                  possible_values: vec![ObjectPropertyValue::Field(value, false)],
                  non_existent: ConsumableCollector::default(),
                },
              );
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }
    } else {
      indeterminate = true;

      let mut unknown_keyed = self.unknown_keyed.borrow_mut();
      unknown_keyed.possible_values.push(ObjectPropertyValue::Field(value, false));

      let mut string_keyed = self.string_keyed.borrow_mut();
      for property in string_keyed.values_mut() {
        property.set(true, value, &mut setters);
      }

      if let Some(rest) = &mut *self.rest.borrow_mut() {
        rest.set(true, value, &mut setters);
      }
    }

    if setters.len() > 0 {
      let indeterminate = indeterminate || setters.len() > 1 || setters[0].0;
      analyzer.push_cf_scope_with_deps(
        CfScopeKind::Dependent,
        None,
        vec![box_consumable((dep, key))],
        if indeterminate { None } else { Some(false) },
      );
      for (_, call_dep, setter) in setters {
        setter.call_as_setter(analyzer, box_consumable(call_dep), rc, value);
      }
      analyzer.pop_cf_scope();
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

    analyzer.mark_object_property_exhaustive_read(self.cf_scope, self.object_id);
    analyzer.push_indeterminate_cf_scope();

    let mut result = vec![];
    let mut non_existent = vec![];

    {
      let mut values = vec![];
      let mut getters = vec![];

      {
        let mut unknown_keyed = self.unknown_keyed.borrow_mut();
        unknown_keyed.get(analyzer, &mut values, &mut getters, &mut non_existent);
        if let Some(rest) = &mut *self.rest.borrow_mut() {
          rest.get(analyzer, &mut values, &mut getters, &mut non_existent);
        }
      }

      for getter in getters {
        values.push(getter.call_as_getter(analyzer, dep.cloned(), rc));
      }

      if let Some(value) = analyzer.factory.try_union(values) {
        result.push((false, analyzer.factory.unknown_primitive, value));
      }
    }

    {
      let string_keyed = self.string_keyed.borrow();
      let keys = string_keyed.keys().cloned().collect::<Vec<_>>();
      mem::drop(string_keyed);
      for key in keys {
        let mut string_keyed = self.string_keyed.borrow_mut();
        let properties = string_keyed.get_mut(&key).unwrap();

        let definite = properties.definite;
        let mut values = vec![];
        let mut getters = vec![];
        properties.get(analyzer, &mut values, &mut getters, &mut non_existent);
        mem::drop(string_keyed);

        for getter in getters {
          values.push(getter.call_as_getter(analyzer, dep.cloned(), rc));
        }

        if let Some(value) = analyzer.factory.try_union(values) {
          result.push((definite, analyzer.factory.string(key), value));
        }
      }
    }

    analyzer.pop_cf_scope();

    (result, box_consumable(ConsumableNode::new((dep, non_existent))))
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    if self.consumed.get() {
      return consumed_object::delete_property(analyzer, dep, key);
    }

    let (has_exhaustive, indeterminate, exec_deps) =
      analyzer.pre_mutate_object(self.cf_scope, self.object_id);

    if has_exhaustive {
      self.consume(analyzer);
      return consumed_object::delete_property(analyzer, dep, key);
    }

    let key = key.get_to_property_key(analyzer);
    let dep = (dep, exec_deps);

    if let Some(key_literals) = key.get_to_literals(analyzer) {
      let indeterminate = indeterminate || key_literals.len() > 1;

      let mut string_keyed = self.string_keyed.borrow_mut();
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(property) = string_keyed.get_mut(key) {
              property.delete(indeterminate, dep.cloned());
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }
    } else {
      let mut string_keyed = self.string_keyed.borrow_mut();
      for property in string_keyed.values_mut() {
        property.delete(true, dep.cloned());
      }
    }

    let mut unknown_keyed = self.unknown_keyed.borrow_mut();
    if !unknown_keyed.possible_values.is_empty() {
      unknown_keyed.delete(true, dep.cloned());
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
      // deps: Default::default(),
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
                  for property in existing.possible_values.iter() {
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
              PropertyKind::Init => ObjectPropertyValue::Field(value.clone(), false),
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
              let property = ObjectProperty {
                definite,
                possible_values: vec![property_val],
                non_existent: ConsumableCollector::default(),
              };
              string_keyed.insert(key, property);
            } else {
              existing.unwrap().possible_values.push(property_val);
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }
    } else {
      let property_val = match kind {
        PropertyKind::Init => ObjectPropertyValue::Field(value.clone(), false),
        PropertyKind::Get => ObjectPropertyValue::Property(Some(value.clone()), None),
        PropertyKind::Set => ObjectPropertyValue::Property(None, Some(value.clone())),
      };
      self.unknown_keyed.borrow_mut().possible_values.push(property_val);
    }
  }

  pub fn init_spread(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    argument: Entity<'a>,
  ) {
    let (properties, deps) = argument.enumerate_properties(analyzer, dep);
    for (definite, key, value) in properties {
      self.init_property(analyzer, PropertyKind::Init, key, value, definite);
    }
    self.unknown_keyed.borrow_mut().non_existent.push(deps);
  }

  pub fn init_rest(&self, property: ObjectPropertyValue<'a>) {
    let mut rest = self.rest.borrow_mut();
    if let Some(rest) = &mut *rest {
      rest.possible_values.push(property);
    } else {
      *rest = Some(ObjectProperty {
        definite: false,
        possible_values: vec![property],
        non_existent: ConsumableCollector::default(),
      });
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_empty_object(&mut self, prototype: &'a Prototype<'a>) -> ObjectEntity<'a> {
    ObjectEntity {
      consumable: true,
      consumed: Cell::new(false),
      // deps: Default::default(),
      cf_scope: self.scope_context.cf.current_id(),
      object_id: self.scope_context.alloc_object_id(),
      string_keyed: RefCell::new(FxHashMap::default()),
      unknown_keyed: RefCell::new(ObjectProperty::default()),
      rest: RefCell::new(None),
      prototype,
    }
  }
}
