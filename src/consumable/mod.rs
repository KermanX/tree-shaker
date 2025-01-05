mod collector;
mod impls;
mod lazy;
mod once;

use crate::{
  analyzer::Analyzer,
  entity::{Entity, EntityFactory},
  utils::{ast::AstKind2, dep_id::DepId},
};
pub use collector::*;
pub use lazy::*;
use once::OnceConsumable;
use std::fmt::Debug;

pub trait ConsumableTrait<'a>: Debug {
  fn consume(&self, analyzer: &mut Analyzer<'a>);
}

#[derive(Debug, Clone, Copy)]
pub enum Consumable<'a> {
  Dyn(&'a (dyn ConsumableTrait<'a> + 'a)),
  AstKind2(AstKind2<'a>),
  DepId(DepId),
  Entity(Entity<'a>),
}

impl<'a> ConsumableTrait<'a> for Consumable<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    match *self {
      Consumable::Dyn(dep) => dep.consume(analyzer),
      Consumable::AstKind2(dep) => dep.consume(analyzer),
      Consumable::DepId(dep) => dep.consume(analyzer),
      Consumable::Entity(dep) => dep.consume(analyzer),
    }
  }
}

impl<'a> From<AstKind2<'a>> for Consumable<'a> {
  fn from(value: AstKind2<'a>) -> Self {
    Self::AstKind2(value)
  }
}

impl<'a> From<DepId> for Consumable<'a> {
  fn from(value: DepId) -> Self {
    Self::DepId(value)
  }
}

impl<'a> From<Entity<'a>> for Consumable<'a> {
  fn from(value: Entity<'a>) -> Self {
    Self::Entity(value)
  }
}

pub type ConsumableVec<'a> = Vec<Consumable<'a>>;

impl<'a> EntityFactory<'a> {
  pub fn consumable_no_once(&self, dep: impl ConsumableTrait<'a> + 'a) -> Consumable<'a> {
    Consumable::Dyn(self.alloc(dep))
  }

  pub fn consumable_once(&self, dep: impl ConsumableTrait<'a> + 'a) -> Consumable<'a> {
    self.consumable_no_once(OnceConsumable::new(dep))
  }

  pub fn consumable(&self, dep: impl ConsumableTrait<'a> + 'a) -> Consumable<'a> {
    self.consumable_once(dep)
  }
}

impl<'a> Analyzer<'a> {
  pub fn consume(&mut self, dep: impl ConsumableTrait<'a> + 'a) {
    dep.consume(self);
  }

  pub fn consumable(&self, dep: impl ConsumableTrait<'a> + 'a) -> Consumable<'a> {
    self.factory.consumable(dep)
  }
}
