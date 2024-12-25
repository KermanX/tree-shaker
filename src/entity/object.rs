use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  builtins::Prototype,
  consumable::{box_consumable, Consumable, ConsumableCollector, ConsumableNode, ConsumableTrait},
  dep::DepId,
  mangling::{is_literal_mangable, MangleAtom, MangleConstraint, UniquenessGroupId},
  scope::CfScopeKind,
  use_consumed_flag,
};
use oxc::{
  ast::ast::PropertyKind,
  semantic::{ScopeId, SymbolId},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
  cell::{Cell, RefCell},
  mem,
};

type ObjectManglingGroupId<'a> = &'a Cell<Option<UniquenessGroupId>>;

#[derive(Debug)]
pub struct ObjectEntity<'a> {
  /// A built-in object is usually non-consumable
  pub consumable: bool,
  consumed: Cell<bool>,
  // deps: RefCell<ConsumableCollector<'a>>,
  /// Where the object is created
  cf_scope: ScopeId,
  pub object_id: SymbolId,
  pub prototype: &'a Prototype<'a>,
  /// `None` if not mangable
  /// `Some(None)` if mangable at the beginning, but disabled later
  pub mangling_group: Option<ObjectManglingGroupId<'a>>,

  /// Properties keyed by known string
  pub string_keyed: RefCell<FxHashMap<&'a str, ObjectProperty<'a>>>,
  /// Properties keyed by unknown value
  pub unknown_keyed: RefCell<ObjectProperty<'a>>,
  /// Properties keyed by unknown value, but not included in `string_keyed`
  pub rest: RefCell<Option<ObjectProperty<'a>>>,
  // TODO: symbol_keyed
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
  pub definite: bool,                                // 是否一定存在
  pub possible_values: Vec<ObjectPropertyValue<'a>>, // 可能的值，可能有多个
  pub non_existent: ConsumableCollector<'a>,         // 如果不存在，为什么？
  pub mangling: Option<(Entity<'a>, MangleAtom)>,
}

impl<'a> Default for ObjectProperty<'a> {
  fn default() -> Self {
    Self {
      definite: true,
      possible_values: vec![],
      non_existent: ConsumableCollector::default(),
      mangling: None,
    }
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

  pub fn get_mangable(
    &mut self,
    analyzer: &Analyzer<'a>,
    values: &mut Vec<Entity<'a>>,
    getters: &mut Vec<Entity<'a>>,
    non_existent: &mut Vec<ConsumableNode<'a>>,
    key: Entity<'a>,
    key_atom: MangleAtom,
  ) {
    let (prev_key, prev_atom) = self.mangling.unwrap();
    let constraint = analyzer.factory.alloc(MangleConstraint::Eq(prev_atom, key_atom));
    for possible_value in &self.possible_values {
      match possible_value {
        ObjectPropertyValue::Field(value, _) => {
          values.push(analyzer.factory.mangable(*value, (prev_key, key), constraint))
        }
        ObjectPropertyValue::Property(Some(getter), _) => getters.push(*getter),
        ObjectPropertyValue::Property(None, _) => values.push(analyzer.factory.mangable(
          analyzer.factory.undefined,
          (prev_key, key),
          constraint,
        )),
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

    self.disable_mangling(analyzer);

    for property in self.string_keyed.take().into_values() {
      property.consume(analyzer);
    }
    self.unknown_keyed.take().consume(analyzer);

    analyzer.mark_object_consumed(self.cf_scope, self.object_id);
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    if self.consumed.get() {
      return consumed_object::unknown_mutate(analyzer, dep);
    }

    self.unknown_keyed.borrow_mut().non_existent.push(dep);
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

    let mut mangable = false;
    let mut values = vec![];
    let mut getters = vec![];
    let mut non_existent = vec![];

    let mut check_rest = false;
    let mut may_add_undefined = false;
    let key = key.get_to_property_key(analyzer);
    if let Some(key_literals) = key.get_to_literals(analyzer) {
      mangable = self.check_mangable(analyzer, &key_literals);
      let mut string_keyed = self.string_keyed.borrow_mut();
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key_str, key_atom) => {
            if let Some(property) = string_keyed.get_mut(key_str) {
              if mangable {
                property.get_mangable(
                  analyzer,
                  &mut values,
                  &mut getters,
                  &mut non_existent,
                  key,
                  key_atom.unwrap(),
                );
              } else {
                property.get(analyzer, &mut values, &mut getters, &mut non_existent);
              }
            } else {
              check_rest = true;
              if let Some(val) = self.prototype.get_string_keyed(key_str) {
                values.push(if mangable { analyzer.factory.computed(val, key_atom) } else { val });
              } else {
                may_add_undefined = true;
              }
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }

      check_rest |= !non_existent.is_empty();
      may_add_undefined |= !non_existent.is_empty();
    } else {
      self.disable_mangling(analyzer);

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

    let indeterminate_getter = !values.is_empty() || getters.len() > 1 || !non_existent.is_empty();

    {
      let mut unknown_keyed = self.unknown_keyed.borrow_mut();
      unknown_keyed.get(analyzer, &mut values, &mut getters, &mut non_existent);
    }

    if !getters.is_empty() {
      if indeterminate_getter {
        analyzer.push_indeterminate_cf_scope();
      }
      for getter in getters {
        // TODO: Support mangling
        values.push(getter.call_as_getter(analyzer, box_consumable((dep.cloned(), key)), rc));
      }
      if indeterminate_getter {
        analyzer.pop_cf_scope();
      }
    }

    let value = analyzer.factory.try_union(values).unwrap_or(analyzer.factory.undefined);
    if mangable {
      analyzer.factory.computed(value, ConsumableNode::new((non_existent, dep)))
    } else {
      analyzer.factory.computed(value, ConsumableNode::new((non_existent, dep, key)))
    }
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

    let mut setters = vec![];

    {
      let unknown_keyed = self.unknown_keyed.borrow();
      for possible_value in &unknown_keyed.possible_values {
        if let ObjectPropertyValue::Property(_, setter) = possible_value {
          if let Some(setter) = setter {
            setters.push((true, None, *setter));
          }
          indeterminate = true;
        }
      }
    }

    let key = key.get_to_property_key(analyzer);
    if let Some(key_literals) = key.get_to_literals(analyzer) {
      let mut string_keyed = self.string_keyed.borrow_mut();
      let mut rest = self.rest.borrow_mut();

      indeterminate |= key_literals.len() > 1;

      let mangable = self.check_mangable(analyzer, &key_literals);
      let value = if mangable {
        analyzer.factory.computed(value, (exec_deps, dep.cloned()))
      } else {
        self.disable_mangling(analyzer);
        analyzer.factory.computed(value, (exec_deps, dep.cloned(), key))
      };

      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key_str, key_atom) => {
            if let Some(property) = string_keyed.get_mut(key_str) {
              let value = if mangable {
                let (prev_key, prev_atom) = property.mangling.unwrap();
                analyzer.factory.mangable(
                  value,
                  (prev_key, key),
                  analyzer.factory.alloc(MangleConstraint::Eq(prev_atom, key_atom.unwrap())),
                )
              } else {
                value
              };
              property.set(indeterminate, value, &mut setters);
            } else if let Some(rest) = &mut *rest {
              rest.set(true, value, &mut setters);
            } else {
              if mangable {
                self.add_to_mangling_group(analyzer, key_atom.unwrap());
              }
              string_keyed.insert(
                key_str,
                ObjectProperty {
                  definite: !indeterminate,
                  possible_values: vec![ObjectPropertyValue::Field(value, false)],
                  non_existent: ConsumableCollector::default(),
                  mangling: mangable.then(|| (key, key_atom.unwrap())),
                },
              );
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }
    } else {
      self.disable_mangling(analyzer);

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

    if !setters.is_empty() {
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
      let mangable = self.is_mangable();
      for key in keys {
        let mut string_keyed = self.string_keyed.borrow_mut();
        let properties = string_keyed.get_mut(&key).unwrap();

        let definite = properties.definite;
        let key_entity = if mangable {
          analyzer.factory.mangable_string(key, properties.mangling.unwrap().1)
        } else {
          analyzer.factory.string(key)
        };

        let mut values = vec![];
        let mut getters = vec![];
        properties.get(analyzer, &mut values, &mut getters, &mut non_existent);
        mem::drop(string_keyed);
        for getter in getters {
          values.push(getter.call_as_getter(analyzer, dep.cloned(), rc));
        }

        if let Some(value) = analyzer.factory.try_union(values) {
          result.push((definite, key_entity, value));
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

    {
      let mut unknown_keyed = self.unknown_keyed.borrow_mut();
      if !unknown_keyed.possible_values.is_empty() {
        unknown_keyed.delete(true, box_consumable((exec_deps.clone(), dep.cloned(), key)));
      }
    }

    if let Some(key_literals) = key.get_to_literals(analyzer) {
      let indeterminate = indeterminate || key_literals.len() > 1;
      let mangable = self.check_mangable(analyzer, &key_literals);
      let dep = if mangable {
        box_consumable((exec_deps, dep))
      } else {
        box_consumable((exec_deps, dep, key))
      };

      let mut string_keyed = self.string_keyed.borrow_mut();
      let mut rest = self.rest.borrow_mut();
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key_str, key_atom) => {
            if let Some(property) = string_keyed.get_mut(key_str) {
              property.delete(
                indeterminate,
                if mangable {
                  let (prev_key, prev_atom) = property.mangling.unwrap();
                  box_consumable((
                    dep.cloned(),
                    // This is a hack
                    analyzer.factory.mangable(
                      analyzer.factory.immutable_unknown,
                      (prev_key, key),
                      analyzer.factory.alloc(MangleConstraint::Eq(prev_atom, key_atom.unwrap())),
                    ),
                  ))
                } else {
                  dep.cloned()
                },
              );
            } else if let Some(rest) = &mut *rest {
              rest.delete(true, box_consumable((dep.cloned(), key)));
            } else if mangable {
              self.add_to_mangling_group(analyzer, key_atom.unwrap());
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }
    } else {
      self.disable_mangling(analyzer);

      let dep = box_consumable((exec_deps, dep, key));

      let mut string_keyed = self.string_keyed.borrow_mut();
      for property in string_keyed.values_mut() {
        property.delete(true, dep.cloned());
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
      // deps: Default::default(),
      cf_scope: ScopeId::new(0),
      object_id,
      string_keyed: Default::default(),
      unknown_keyed: Default::default(),
      rest: Default::default(),
      prototype,
      mangling_group: None,
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
    if let Some(key_literals) = key.get_to_literals(analyzer) {
      let mangable = self.check_mangable(analyzer, &key_literals);
      let value = if mangable { value } else { analyzer.factory.computed(value, key) };

      let definite = definite && key_literals.len() == 1;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key_str, key_atom) => {
            let mut string_keyed = self.string_keyed.borrow_mut();
            let existing = string_keyed.get_mut(key_str);
            let reused_property = definite
              .then(|| {
                existing.as_ref().and_then(|existing| {
                  for property in existing.possible_values.iter() {
                    if let ObjectPropertyValue::Property(getter, setter) = property {
                      return Some((*getter, *setter));
                    }
                  }
                  None
                })
              })
              .flatten();
            let constraint = if mangable {
              if let Some(existing) = &existing {
                let (_, existing_atom) = existing.mangling.unwrap();
                Some(MangleConstraint::Eq(existing_atom, key_atom.unwrap()))
              } else {
                self.add_to_mangling_group(analyzer, key_atom.unwrap());
                None
              }
            } else {
              None
            };
            let value = if let Some(constraint) = constraint {
              analyzer.factory.computed(value, &*analyzer.factory.alloc(constraint))
            } else {
              value
            };
            let property_val = match kind {
              PropertyKind::Init => ObjectPropertyValue::Field(value, false),
              PropertyKind::Get => ObjectPropertyValue::Property(
                Some(value),
                reused_property.and_then(|(_, setter)| setter),
              ),
              PropertyKind::Set => ObjectPropertyValue::Property(
                reused_property.and_then(|(getter, _)| getter),
                Some(value),
              ),
            };
            let existing = string_keyed.get_mut(key_str);
            if definite || existing.is_none() {
              let property = ObjectProperty {
                definite,
                possible_values: vec![property_val],
                non_existent: ConsumableCollector::default(),
                mangling: mangable.then(|| (key, key_atom.unwrap())),
              };
              string_keyed.insert(key_str, property);
            } else {
              existing.unwrap().possible_values.push(property_val);
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }
    } else {
      self.disable_mangling(analyzer);
      let value = analyzer.factory.computed(value, key);
      let property_val = match kind {
        PropertyKind::Init => ObjectPropertyValue::Field(value, false),
        PropertyKind::Get => ObjectPropertyValue::Property(Some(value), None),
        PropertyKind::Set => ObjectPropertyValue::Property(None, Some(value)),
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
    debug_assert_eq!(self.mangling_group, None);
    let mut rest = self.rest.borrow_mut();
    if let Some(rest) = &mut *rest {
      rest.possible_values.push(property);
    } else {
      *rest = Some(ObjectProperty {
        definite: false,
        possible_values: vec![property],
        non_existent: ConsumableCollector::default(),
        mangling: None,
      });
    }
  }

  fn is_mangable(&self) -> bool {
    self.mangling_group.is_some_and(|group| group.get().is_some())
  }

  fn check_mangable(
    &self,
    analyzer: &mut Analyzer<'a>,
    literals: &FxHashSet<LiteralEntity>,
  ) -> bool {
    if self.is_mangable() {
      if is_literal_mangable(literals) {
        true
      } else {
        self.disable_mangling(analyzer);
        false
      }
    } else {
      false
    }
  }

  fn disable_mangling(&self, analyzer: &mut Analyzer<'a>) {
    if let Some(group) = self.mangling_group {
      if let Some(group) = group.replace(None) {
        analyzer.mangler.mark_uniqueness_group_non_mangable(group);
      }
    }
  }

  fn add_to_mangling_group(&self, analyzer: &mut Analyzer<'a>, key_atom: MangleAtom) {
    analyzer.mangler.add_to_uniqueness_group(self.mangling_group.unwrap().get().unwrap(), key_atom);
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_empty_object(
    &mut self,
    prototype: &'a Prototype<'a>,
    mangling_group: Option<ObjectManglingGroupId<'a>>,
  ) -> ObjectEntity<'a> {
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
      mangling_group,
    }
  }

  pub fn new_function_object(&mut self) -> &'a ObjectEntity<'a> {
    let object = self.new_empty_object(&self.builtins.prototypes.function, None);
    object.string_keyed.borrow_mut().insert(
      "prototype",
      ObjectProperty {
        definite: true,
        possible_values: vec![ObjectPropertyValue::Field(
          self.factory.entity(self.new_empty_object(&self.builtins.prototypes.object, None)),
          false,
        )],
        non_existent: Default::default(),
        mangling: None,
      },
    );
    self.allocator.alloc(object)
  }

  pub fn new_object_mangling_group(&mut self) -> ObjectManglingGroupId<'a> {
    self.allocator.alloc(Cell::new(Some(self.mangler.uniqueness_groups.push(Default::default()))))
  }

  pub fn use_mangable_plain_object(&mut self, dep_id: impl Into<DepId>) -> ObjectEntity<'a> {
    let mangling_group = self
      .load_data::<Option<ObjectManglingGroupId>>(dep_id)
      .get_or_insert_with(|| self.new_object_mangling_group());
    self.new_empty_object(&self.builtins.prototypes.object, Some(*mangling_group))
  }
}
