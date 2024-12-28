mod delete;
mod enumerate;
mod get;
mod init;
mod property;
mod set;

use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  builtins::Prototype,
  consumable::Consumable,
  dep::DepId,
  mangling::{is_literal_mangable, MangleAtom, UniquenessGroupId},
  use_consumed_flag,
};
use oxc::semantic::{ScopeId, SymbolId};
pub use property::{ObjectProperty, ObjectPropertyValue};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::{Cell, RefCell};

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
    self.get_property(rc, analyzer, dep, key)
  }

  fn set_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.set_property(rc, analyzer, dep, key, value);
  }

  fn enumerate_properties(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    self.enumerate_properties(rc, analyzer, dep)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.delete_property(analyzer, dep, key);
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

  fn get_destructable(
    &self,
    _rc: Entity<'a>,
    _analyzer: &Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Consumable<'a> {
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
