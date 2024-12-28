mod collector;
mod impls;
mod lazy;

pub use collector::*;
pub use lazy::*;

use crate::{analyzer::Analyzer, entity::EntityFactory};
use std::{cell::Cell, fmt::Debug};

pub trait ConsumableTrait<'a>: Debug {
  fn consume(&self, analyzer: &mut Analyzer<'a>);
}

#[derive(Debug, Clone, Copy)]
pub struct Consumable<'a>(pub &'a (dyn ConsumableTrait<'a> + 'a));

pub type ConsumableVec<'a> = Vec<Consumable<'a>>;

impl<'a> EntityFactory<'a> {
  pub fn consumable(&self, dep: impl ConsumableTrait<'a> + 'a) -> Consumable<'a> {
    Consumable(self.alloc(Cell::new(Some(&*self.alloc(dep)))))
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
