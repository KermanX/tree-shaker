use super::{
  consumed_object,
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use crate::analyzer::Analyzer;
use std::fmt::Debug;

pub trait BuiltinFnEntity<'a>: Debug {
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a>;
}

impl<'a, T: BuiltinFnEntity<'a>> EntityTrait<'a> for T {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}

  fn consume_as_unknown(&self, _analyzer: &mut Analyzer<'a>) {}

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    analyzer.builtins.prototypes.function.get_property(key, dep)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    // TODO: throw warning
    consumed_object::set_property(analyzer, dep, key, value)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep, key: &Entity<'a>) {
    // TODO: throw warning
    consumed_object::delete_property(analyzer, dep, key)
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    vec![]
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.call_impl(analyzer, dep, this, args)
  }

  fn r#await(&self, rc: &Entity<'a>, _analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    (false, rc.clone())
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    // TODO: throw warning
    consumed_object::iterate(analyzer, dep)
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("function")
  }

  fn get_to_string(&self, rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![rc.clone()])
  }

  fn get_to_property_key(&self, rc: &Entity<'a>) -> Entity<'a> {
    self.get_to_string(rc)
  }

  fn test_typeof(&self) -> TypeofResult {
    TypeofResult::Function
  }

  fn test_truthy(&self) -> Option<bool> {
    Some(true)
  }

  fn test_nullish(&self) -> Option<bool> {
    Some(false)
  }
}

pub type BuiltinFnImplementation<'a> =
  fn(&mut Analyzer<'a>, &Entity<'a>, &Entity<'a>) -> Entity<'a>;

#[derive(Debug, Clone, Copy)]
pub struct ImplementedBuiltinFnEntity<'a> {
  implementation: BuiltinFnImplementation<'a>,
}

impl<'a> BuiltinFnEntity<'a> for ImplementedBuiltinFnEntity<'a> {
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    _dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    (self.implementation)(analyzer, this, args)
  }
}

impl<'a> ImplementedBuiltinFnEntity<'a> {
  pub fn new(implementation: BuiltinFnImplementation<'a>) -> Entity<'a> {
    Entity::new(Self { implementation })
  }
}

#[derive(Debug, Clone)]
pub struct PureBuiltinFnEntity<'a> {
  return_value: Entity<'a>,
}

impl<'a> BuiltinFnEntity<'a> for PureBuiltinFnEntity<'a> {
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    _dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    this.consume_as_unknown(analyzer);
    args.consume_as_unknown(analyzer);
    self.return_value.clone()
  }
}

impl<'a> PureBuiltinFnEntity<'a> {
  pub fn new(return_value: Entity<'a>) -> Entity<'a> {
    Entity::new(Self { return_value })
  }

  pub fn returns_unknown_entity(kind: UnknownEntityKind) -> Entity<'a> {
    Entity::new(Self { return_value: UnknownEntity::new(kind) })
  }

  pub fn returns_unknown() -> Entity<'a> {
    Self::returns_unknown_entity(UnknownEntityKind::Unknown)
  }

  pub fn returns_string() -> Entity<'a> {
    Self::returns_unknown_entity(UnknownEntityKind::String)
  }

  pub fn returns_number() -> Entity<'a> {
    Self::returns_unknown_entity(UnknownEntityKind::Number)
  }

  pub fn returns_boolean() -> Entity<'a> {
    Self::returns_unknown_entity(UnknownEntityKind::Boolean)
  }

  pub fn returns_array() -> Entity<'a> {
    Self::returns_unknown_entity(UnknownEntityKind::Object)
  }

  pub fn returns_null() -> Entity<'a> {
    Self::new(LiteralEntity::new_null())
  }

  pub fn returns_undefined() -> Entity<'a> {
    Self::new(LiteralEntity::new_undefined())
  }

  pub fn returns_object() -> Entity<'a> {
    Self::returns_unknown_entity(UnknownEntityKind::Object)
  }
}
