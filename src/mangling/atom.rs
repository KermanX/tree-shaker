use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableTrait},
};
use std::{
  num::{NonZero, NonZeroU32},
  sync::atomic::{AtomicU32, Ordering},
};

static MANGLE_ATOM_COUNT: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MangleAtom(NonZeroU32);

impl MangleAtom {
  pub fn new(_info: impl std::fmt::Debug) -> MangleAtom {
    let id = MANGLE_ATOM_COUNT.fetch_add(1, Ordering::SeqCst);
    // println!("{_info:?} -> MangleAtom({})", id);
    MangleAtom(NonZero::new(id).unwrap())
  }
}

impl<'a> ConsumableTrait<'a> for MangleAtom {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    analyzer.mangler.non_mangable.insert(*self);
  }

  fn cloned(&self) -> Consumable<'a> {
    Box::new(*self)
  }
}
