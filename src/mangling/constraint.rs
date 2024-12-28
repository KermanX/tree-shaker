use super::{AtomState, MangleAtom};
use super::{Mangler, UniquenessGroupId};
use crate::utils::get_two_mut_from_vec;
use crate::{analyzer::Analyzer, consumable::ConsumableTrait};
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

  pub fn add_to_mangler(&self, mangler: &mut Mangler) {
    match self {
      MangleConstraint::NonMangable(a) => {
        mangler.mark_atom_non_mangable(*a);
      }
      MangleConstraint::Eq(a, b) => {
        mangler.mark_equality(true, *a, *b);
      }
      MangleConstraint::Neq(a, b) => {
        mangler.mark_equality(false, *a, *b);
      }
      MangleConstraint::Unique(g, a) => {
        mangler.add_to_uniqueness_group(*g, *a);
      }
      MangleConstraint::Multiple(cs) => {
        for constraint in cs {
          constraint.add_to_mangler(mangler);
        }
      }
    }
  }
}

impl<'a> ConsumableTrait<'a> for &'a MangleConstraint {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.add_to_mangler(&mut analyzer.mangler);
  }
}

impl<'a> Mangler<'a> {
  fn mark_equality(&mut self, eq: bool, a: MangleAtom, b: MangleAtom) {
    if a == b {
      return;
    }

    let Mangler { atoms, identity_groups, uniqueness_groups, .. } = self;

    match get_two_mut_from_vec(atoms, a, b) {
      (AtomState::Constant(a), AtomState::Constant(b)) => assert_eq!(a, b),
      (AtomState::Constant(a), _) => {
        let s = *a;
        self.mark_atom_constant(b, s);
      }
      (_, AtomState::Constant(b)) => {
        let s = *b;
        self.mark_atom_constant(a, s);
      }
      (AtomState::NonMangable, AtomState::NonMangable) => {}
      (AtomState::NonMangable, _) => self.mark_atom_non_mangable(b),
      (_, AtomState::NonMangable) => self.mark_atom_non_mangable(a),
      (AtomState::Constrained(ea, ua), AtomState::Constrained(eb, ub)) => {
        if eq {
          match ((ea, ua), (eb, ub)) {
            ((Some(ia), _), (Some(ib), _)) => {
              if ia != ib {
                let ((ga, _), (gb, _)) = get_two_mut_from_vec(identity_groups, *ia, *ib);
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
                  let AtomState::Constrained(group, _) = &mut atoms[atom] else {
                    unreachable!();
                  };
                  *group = Some(index);
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
          ua.insert(id);
          ub.insert(id);
        }
      }
    }
  }

  pub fn mark_atom_non_mangable(&mut self, atom: MangleAtom) {
    let state = &mut self.atoms[atom];
    if matches!(state, AtomState::Constant(_)) {
      return;
    }
    if let AtomState::Constrained(identity_group, uniqueness_groups) =
      mem::replace(state, AtomState::NonMangable)
    {
      if let Some(index) = identity_group {
        for atom in mem::take(&mut self.identity_groups[index].0) {
          self.mark_atom_non_mangable(atom);
        }
      }
      for index in uniqueness_groups {
        for atom in mem::take(&mut self.uniqueness_groups[index].0) {
          self.mark_atom_non_mangable(atom);
        }
      }
    }
  }

  pub fn mark_uniqueness_group_non_mangable(&mut self, group: UniquenessGroupId) {
    for atom in mem::take(&mut self.uniqueness_groups[group].0) {
      self.mark_atom_non_mangable(atom);
    }
  }

  pub fn add_to_uniqueness_group(&mut self, group: UniquenessGroupId, atom: MangleAtom) {
    match &mut self.atoms[atom] {
      AtomState::Constrained(_, uniqueness_groups) => {
        uniqueness_groups.insert(group);
        self.uniqueness_groups[group].0.push(atom);
      }
      AtomState::Constant(_) => {
        self.uniqueness_groups[group].0.push(atom);
      }
      AtomState::NonMangable => {
        self.mark_uniqueness_group_non_mangable(group);
      }
    }
  }

  fn mark_atom_constant(&mut self, atom: MangleAtom, value: &'a str) {
    let Mangler { identity_groups, atoms, .. } = self;

    let atom = mem::replace(&mut atoms[atom], AtomState::Constant(value));

    if let AtomState::Constrained(Some(identity_group), _) = atom {
      for atom in mem::take(&mut identity_groups[identity_group].0) {
        atoms[atom] = AtomState::Constant(value);
      }
    }
  }
}
