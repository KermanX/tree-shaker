use super::{
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
};
use crate::{analyzer::Analyzer, builtins::Prototype};
use std::cell::RefCell;

#[derive(Debug, Clone, Copy)]
pub enum UnknownEntityKind {
  // TODO: NumericString, NoneEmptyString, ...
  String,
  Number,
  BigInt,
  Boolean,
  Symbol,
  Function,
  Regexp,
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

  fn get_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> (bool, Entity<'a>) {
    if matches!(self.kind, UnknownEntityKind::Unknown) {
      self.consume_as_unknown(analyzer);
      key.get_to_property_key().consume_self(analyzer);
      (true, UnknownEntity::new_unknown())
    } else {
      let prototype = self.get_prototype(analyzer);
      prototype.get_property(key)
    }
  }

  fn set_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool {
    if self.maybe_object() {
      self.consume_as_unknown(analyzer);
      key.get_to_property_key().consume_self(analyzer);
      value.consume_as_unknown(analyzer);
      true
    } else {
      false
    }
  }

  fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
    if self.maybe_object() {
      self.consume_as_unknown(analyzer);
      UnknownEntity::new_unknown_to_entries_result(self.deps.borrow().clone())
    } else {
      (false, vec![])
    }
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
    if self.maybe_object() {
      key.get_to_property_key().consume_self(analyzer);
      self.consume_as_unknown(analyzer);
      true
    } else {
      false
    }
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    if !self.maybe_object() {
      // TODO: throw warning
    }
    self.consume_as_unknown(analyzer);
    this.consume_as_unknown(analyzer);
    args.consume_as_unknown(analyzer);
    (true, UnknownEntity::new_unknown())
  }

  fn r#await(&self, rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    if self.maybe_object() {
      self.consume_as_unknown(analyzer);
      (true, UnknownEntity::new_unknown())
    } else {
      (false, rc.clone())
    }
  }

  fn iterate(&self, _rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Option<Entity<'a>>) {
    if !self.maybe_object() {
      // TODO: throw warning
    }
    self.consume_as_unknown(analyzer);
    (true, Some(UnknownEntity::new_unknown()))
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

  fn get_to_array(&self, _rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    UnknownEntity::new_unknown_to_array_result(length, self.deps.borrow().clone())
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
      UnknownEntityKind::Object => TypeofResult::Object,
      UnknownEntityKind::Unknown => TypeofResult::_Unknown,
    }
  }

  fn test_truthy(&self) -> Option<bool> {
    match &self.kind {
      UnknownEntityKind::Symbol | UnknownEntityKind::Function | UnknownEntityKind::Object => {
        Some(true)
      }
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

  pub fn new_unknown_to_array_result(
    length: usize,
    deps: Vec<Entity<'a>>,
  ) -> (Vec<Entity<'a>>, Entity<'a>) {
    let mut result = Vec::new();
    for _ in 0..length {
      result.push(UnknownEntity::new_unknown_with_deps(deps.clone()));
    }
    (result, UnknownEntity::new_unknown_with_deps(deps))
  }

  pub fn new_unknown_to_entries_result(
    deps: Vec<Entity<'a>>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
    (
      true,
      vec![(
        false,
        UnknownEntity::new_unknown_with_deps(deps.clone()),
        UnknownEntity::new_unknown_with_deps(deps),
      )],
    )
  }

  pub fn maybe_object(&self) -> bool {
    matches!(
      self.kind,
      UnknownEntityKind::Object
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
      UnknownEntityKind::Object => &analyzer.builtins.prototypes.object,
      UnknownEntityKind::Unknown => unreachable!(),
    }
  }
}
