mod collector;
mod impls;
mod lazy;
mod node;

pub use collector::*;
pub use lazy::*;
pub use node::*;

use crate::analyzer::Analyzer;
use std::fmt::Debug;

pub trait ConsumableTrait<'a>: Debug {
  fn consume(&self, analyzer: &mut Analyzer<'a>);
  fn cloned(&self) -> Consumable<'a>;
}

pub type Consumable<'a> = Box<dyn ConsumableTrait<'a> + 'a>;

pub type ConsumableVec<'a> = Vec<Consumable<'a>>;

impl<'a> Analyzer<'a> {
  pub fn consume(&mut self, dep: impl ConsumableTrait<'a> + 'a) {
    dep.consume(self);
  }
}

pub fn box_consumable<'a>(value: impl ConsumableTrait<'a> + 'a) -> Consumable<'a> {
  Box::new(value)
}
