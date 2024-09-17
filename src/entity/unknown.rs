use super::{
  consumed_object,
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
};
use crate::{analyzer::Analyzer, builtins::Prototype};
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownEntityKind {
  // TODO: NumericString, NoneEmptyString, ...
  String,
  Number,
  BigInt,
  Boolean,
  Symbol,
  Function,
  Regexp,
  Array,
  Object,
  Unknown,
}

#[derive(Debug, Clone)]
pub struct UnknownEntity<'a> {
  pub kind: UnknownEntityKind,
  pub deps: RefCell<Vec<Entity<'a>>>,
}

impl<'a> EntityTrait<'a> for UnknownEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {
    self.consume_as_unknown(analyzer);
  }

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    let mut deps = self.deps.borrow_mut();
    for dep in deps.iter() {
      dep.consume_as_unknown(analyzer);
    }
    deps.clear();
  }

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    if matches!(self.kind, UnknownEntityKind::Unknown) {
      self.consume_as_unknown(analyzer);
      consumed_object::get_property(analyzer, dep, key)
    } else {
      let prototype = self.get_prototype(analyzer);
      prototype.get_property(key, dep)
    }
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    if self.maybe_object() {
      self.consume_as_unknown(analyzer);
      consumed_object::set_property(analyzer, dep, key, value)
    } else {
      // Primitives. No effect
    }
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    if self.maybe_object() {
      self.consume_as_unknown(analyzer);
      consumed_object::enumerate_properties(analyzer, dep)
    } else {
      vec![]
    }
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep, key: &Entity<'a>) {
    if self.maybe_object() {
      consumed_object::delete_property(analyzer, dep, key)
    } else {
      // No effect
    }
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    if !self.maybe_object() {
      // TODO: throw warning
    }
    self.consume_as_unknown(analyzer);
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(&self, rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    if self.maybe_object() {
      self.consume_as_unknown(analyzer);
      (true, UnknownEntity::new_unknown())
    } else {
      (false, rc.clone())
    }
  }

  fn iterate(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    if self.kind == UnknownEntityKind::Array {
      return (vec![], Some(UnknownEntity::new_unknown_with_deps(vec![rc.clone()])));
    }
    if !self.maybe_object() {
      // TODO: throw warning
    }
    self.consume_as_unknown(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_typeof(&self) -> Entity<'a> {
    if let Some(str) = self.test_typeof().to_string() {
      LiteralEntity::new_string(str)
    } else {
      UnknownEntity::new_with_deps(UnknownEntityKind::String, self.deps.borrow().clone())
    }
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_with_deps(UnknownEntityKind::String, self.deps.borrow().clone())
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_with_deps(UnknownEntityKind::Unknown, self.deps.borrow().clone())
  }

  fn test_typeof(&self) -> TypeofResult {
    match &self.kind {
      UnknownEntityKind::String => TypeofResult::String,
      UnknownEntityKind::Number => TypeofResult::Number,
      UnknownEntityKind::BigInt => TypeofResult::BigInt,
      UnknownEntityKind::Boolean => TypeofResult::Boolean,
      UnknownEntityKind::Symbol => TypeofResult::Symbol,
      UnknownEntityKind::Function => TypeofResult::Function,
      UnknownEntityKind::Regexp => TypeofResult::Object,
      UnknownEntityKind::Array => TypeofResult::Object,
      UnknownEntityKind::Object => TypeofResult::Object,
      UnknownEntityKind::Unknown => TypeofResult::_Unknown,
    }
  }

  fn test_truthy(&self) -> Option<bool> {
    match &self.kind {
      UnknownEntityKind::Symbol
      | UnknownEntityKind::Function
      | UnknownEntityKind::Array
      | UnknownEntityKind::Object => Some(true),
      _ => None,
    }
  }

  fn test_nullish(&self) -> Option<bool> {
    match &self.kind {
      UnknownEntityKind::Unknown => None,
      _ => Some(false),
    }
  }

  fn test_is_completely_unknown(&self) -> bool {
    matches!(self.kind, UnknownEntityKind::Unknown) && self.deps.borrow().is_empty()
  }
}

impl<'a> UnknownEntity<'a> {
  pub fn new_with_deps(kind: UnknownEntityKind, deps: Vec<Entity<'a>>) -> Entity<'a> {
    Entity::new(Self { kind, deps: RefCell::new(deps) })
  }

  pub fn new(kind: UnknownEntityKind) -> Entity<'a> {
    Self::new_with_deps(kind, Vec::new())
  }

  pub fn new_unknown() -> Entity<'a> {
    Self::new(UnknownEntityKind::Unknown)
  }

  pub fn new_unknown_with_deps(deps: Vec<Entity<'a>>) -> Entity<'a> {
    Self::new_with_deps(UnknownEntityKind::Unknown, deps)
  }

  pub fn maybe_object(&self) -> bool {
    matches!(
      self.kind,
      UnknownEntityKind::Object
        | UnknownEntityKind::Array
        | UnknownEntityKind::Function
        | UnknownEntityKind::Regexp
        | UnknownEntityKind::Unknown
    )
  }

  fn get_prototype<'b>(&self, analyzer: &'b mut Analyzer<'a>) -> &'b Prototype<'a> {
    match &self.kind {
      UnknownEntityKind::String => &analyzer.builtins.prototypes.string,
      UnknownEntityKind::Number => &analyzer.builtins.prototypes.number,
      UnknownEntityKind::BigInt => &analyzer.builtins.prototypes.bigint,
      UnknownEntityKind::Boolean => &analyzer.builtins.prototypes.boolean,
      UnknownEntityKind::Symbol => &analyzer.builtins.prototypes.symbol,
      UnknownEntityKind::Function => &analyzer.builtins.prototypes.function,
      UnknownEntityKind::Regexp => &analyzer.builtins.prototypes.regexp,
      UnknownEntityKind::Array => &analyzer.builtins.prototypes.array,
      UnknownEntityKind::Object => &analyzer.builtins.prototypes.object,
      UnknownEntityKind::Unknown => unreachable!(),
    }
  }
}
