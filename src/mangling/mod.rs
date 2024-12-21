use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableTrait},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
  num::{NonZero, NonZeroU64},
  rc::Rc,
  sync::atomic::{AtomicU64, Ordering},
};

static MANGLE_ATOM_COUNT: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MangleAtom(NonZeroU64);

impl<'a> MangleAtom {
  pub fn new() -> MangleAtom {
    MangleAtom(NonZero::new(MANGLE_ATOM_COUNT.fetch_add(1, Ordering::SeqCst)).unwrap())
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

#[derive(Debug, Clone)]
pub enum MangleConstraint {
  Uniqueness(Rc<Vec<MangleAtom>>),
  Equality(MangleAtom, MangleAtom),
}

impl<'a> ConsumableTrait<'a> for MangleConstraint {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    match self {
      MangleConstraint::Uniqueness(atoms) => {
        analyzer.mangler.uniqueness.insert(Rc::clone(atoms));
      }
      MangleConstraint::Equality(a, b) => {
        analyzer.mangler.equality.entry(*a).or_default().push(*b);
      }
    }
  }

  fn cloned(&self) -> Consumable<'a> {
    Box::new(self.clone())
  }
}

#[derive(Debug, Default)]
pub struct Mangler {
  pub non_mangable: FxHashSet<MangleAtom>,
  uniqueness: FxHashSet<Rc<Vec<MangleAtom>>>,
  equality: FxHashMap<MangleAtom, Vec<MangleAtom>>,
}

impl Mangler {
  pub fn new() -> Self {
    Self::default()
  }
}
