use super::{
  consumed_object, Consumable, Entity, EntityTrait, InteractionKind, LiteralEntity, TypeofResult,
  UnknownEntity,
};
use crate::analyzer::Analyzer;
use std::fmt::Debug;

pub trait BuiltinFnEntity<'a>: Debug {
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a>;
}

impl<'a, T: BuiltinFnEntity<'a>> EntityTrait<'a> for T {
  fn consume(&self, _analyzer: &mut Analyzer<'a>) {}

  fn get_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    analyzer.builtins.prototypes.function.get_property(rc, key, dep)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    // TODO: throw warning
    consumed_object::set_property(analyzer, dep, key, value)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: &Entity<'a>) {
    // TODO: throw warning
    consumed_object::delete_property(analyzer, dep, key)
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    vec![]
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.call_impl(analyzer, dep, this, args)
  }

  fn r#await(
    &self,
    rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
  ) -> Entity<'a> {
    rc.clone()
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    // TODO: throw warning
    analyzer.explicit_throw_unknown();
    consumed_object::iterate(analyzer, dep)
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("function")
  }

  fn get_to_string(&self, rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_computed_string(rc.clone())
  }

  fn get_to_numeric(&self, _rc: &Entity<'a>) -> Entity<'a> {
    LiteralEntity::new_nan()
  }

  fn get_to_boolean(&self, rc: &Entity<'a>) -> Entity<'a> {
    LiteralEntity::new_boolean(true)
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
  fn(&mut Analyzer<'a>, Consumable<'a>, &Entity<'a>, &Entity<'a>) -> Entity<'a>;

#[derive(Debug, Clone, Copy)]
pub struct ImplementedBuiltinFnEntity<'a> {
  implementation: BuiltinFnImplementation<'a>,
}

impl<'a> BuiltinFnEntity<'a> for ImplementedBuiltinFnEntity<'a> {
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    (self.implementation)(analyzer, dep, this, args)
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
  interaction_kind: InteractionKind,
}

impl<'a> BuiltinFnEntity<'a> for PureBuiltinFnEntity<'a> {
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    analyzer.consume(dep);
    this.consume(analyzer);
    args.consume(analyzer);
    self.return_value.clone()
  }
}

impl<'a> PureBuiltinFnEntity<'a> {
  pub fn new(return_value: Entity<'a>) -> Self {
    Self { return_value, interaction_kind: InteractionKind::Unknown }
  }

  pub fn interaction_kind(mut self, interaction_kind: InteractionKind) -> Self {
    self.interaction_kind = interaction_kind;
    self
  }

  pub fn returns_unknown() -> Self {
    Self::new(UnknownEntity::new_unknown())
  }

  pub fn returns_string() -> Self {
    Self::new(UnknownEntity::new_string())
  }

  pub fn returns_number() -> Self {
    Self::new(UnknownEntity::new_number())
  }

  pub fn returns_boolean() -> Self {
    Self::new(UnknownEntity::new_boolean())
  }

  pub fn returns_array() -> Self {
    Self::new(UnknownEntity::new_array())
  }

  pub fn returns_object() -> Self {
    Self::new(UnknownEntity::new_object())
  }

  pub fn returns_null() -> Self {
    Self::new(LiteralEntity::new_null())
  }

  pub fn returns_undefined() -> Self {
    Self::new(LiteralEntity::new_undefined())
  }
}
