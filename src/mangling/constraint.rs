use super::MangleAtom;
use super::{Mangler, UniquenessGroupId};
use crate::utils::{get_two_mut_from_map_or_insert, get_two_mut_from_vec_unwrap};
use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableTrait},
};
use std::mem;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MangleConstraint {
  NonMangable(MangleAtom),
  Eq(MangleAtom, MangleAtom),
  Neq(MangleAtom, MangleAtom),
  Unique(UniquenessGroupId, MangleAtom),
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

  pub fn negate_equality(self) -> Self {
    match self {
      MangleConstraint::Eq(a, b) => MangleConstraint::Neq(a, b),
      MangleConstraint::Neq(a, b) => MangleConstraint::Eq(a, b),
      MangleConstraint::Multiple(c) => {
        MangleConstraint::Multiple(c.into_iter().map(Self::negate_equality).collect())
      }
      _ => unreachable!(),
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
      MangleConstraint::Unique(g, a) => {
        let Mangler { uniqueness_groups, atoms, .. } = mangler;
        if atoms.entry(*a).or_default().1.insert(*g) {
          uniqueness_groups[*g].0.push(*a);
        }
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

impl<'a> Mangler<'a> {
  fn mark_equality(&mut self, eq: bool, a: MangleAtom, b: MangleAtom) {
    debug_assert_ne!(a, b);
    let a_is_non_mangable = self.non_mangable.contains(&a);
    let b_is_non_mangable = self.non_mangable.contains(&b);
    if a_is_non_mangable || b_is_non_mangable {
      if !a_is_non_mangable {
        self.mark_atom_non_mangable(a);
      }
      if !b_is_non_mangable {
        self.mark_atom_non_mangable(b);
      }
    } else {
      let Mangler { atoms: constraints, identity_groups, uniqueness_groups, .. } = self;
      let existing = get_two_mut_from_map_or_insert(constraints, a, b);
      if eq {
        match existing {
          ((Some(ia), _), (Some(ib), _)) => {
            if ia != ib {
              let ((ga, _), (gb, _)) = get_two_mut_from_vec_unwrap(identity_groups, *ia, *ib);
              let a_is_larger = ga.len() > gb.len();
              let (from, to) = if a_is_larger {
                *ib = *ia;
                (gb, ga)
              } else {
                *ia = *ib;
                (ga, gb)
              };
              let index = *ia;
              for atom in mem::take(from) {
                to.push(atom);
                constraints.get_mut(&atom).unwrap().0 = Some(index);
              }
            }
          }
          ((Some(ia), _), (ib @ None, _)) => {
            *ib = Some(*ia);
            identity_groups[*ia].0.push(b);
          }
          ((ia @ None, _), (Some(ib), _)) => {
            *ia = Some(*ib);
            identity_groups[*ib].0.push(a);
          }
          ((ia @ None, _), (ib @ None, _)) => {
            let id = identity_groups.push((vec![a, b], None));
            *ia = Some(id);
            *ib = Some(id);
          }
        }
      } else {
        let id = uniqueness_groups.push((vec![a, b], 0));
        let ((_, uniq_groups_a), (_, uniq_groups_b)) = existing;
        uniq_groups_a.insert(id);
        uniq_groups_b.insert(id);
      }
    }
  }
}
