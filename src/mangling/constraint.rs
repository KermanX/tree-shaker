use super::MangleAtom;
use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableTrait},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MangleConstraint {
  NonMangable(MangleAtom),
  Eq(MangleAtom, MangleAtom),
  Neq(MangleAtom, MangleAtom),
  Multiple(Vec<MangleConstraint>),
}

impl MangleConstraint {
  pub fn equality(
    eq: bool,
    a: Option<MangleAtom>,
    b: Option<MangleAtom>,
  ) -> Option<MangleConstraint> {
    if let (Some(a), Some(b)) = (a, b) {
      Some(if eq { MangleConstraint::Eq(a, b) } else { MangleConstraint::Neq(a, b) })
    } else {
      None
    }
  }

  pub fn negate(self) -> Self {
    match self {
      MangleConstraint::NonMangable(a) => MangleConstraint::NonMangable(a),
      MangleConstraint::Eq(a, b) => MangleConstraint::Neq(a, b),
      MangleConstraint::Neq(a, b) => MangleConstraint::Eq(a, b),
      MangleConstraint::Multiple(c) => {
        MangleConstraint::Multiple(c.into_iter().map(Self::negate).collect())
      }
    }
  }
}

impl<'a> ConsumableTrait<'a> for &'a MangleConstraint {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    let mangler = &mut analyzer.mangler;
    match *self {
      MangleConstraint::NonMangable(a) => {
        mangler.non_mangable.insert(*a);
      }
      MangleConstraint::Eq(a, b) => {
        mangler.mark_equality(true, *a, *b);
      }
      MangleConstraint::Neq(a, b) => {
        mangler.mark_equality(false, *a, *b);
      }
      MangleConstraint::Multiple(cs) => {
        for constraint in cs {
          constraint.consume(analyzer);
        }
      }
    }
  }

  fn cloned(&self) -> Consumable<'a> {
    Box::new(*self)
  }
}
