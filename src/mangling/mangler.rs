use super::MangleAtom;
use crate::utils::{get_mangled_name, get_two_mut_from_map_or_insert, get_two_mut_from_vec_unwrap};
use oxc::allocator::Allocator;
use rustc_hash::{FxHashMap, FxHashSet};
use std::mem;

pub type MangleAtomGroups = (Option<usize>, Vec<usize>);

pub struct Mangler<'a> {
  pub allocator: &'a Allocator,

  pub non_mangable: FxHashSet<MangleAtom>,

  /// (atoms, resolved_name)[]
  pub equality_groups: Vec<(Vec<MangleAtom>, Option<&'a str>)>,
  /// (atoms, used_names)[]
  pub uniqueness_groups: Vec<(Vec<MangleAtom>, usize)>,

  /// atom -> (equality_group_index, uniqueness_group_index)
  pub atoms: FxHashMap<MangleAtom, MangleAtomGroups>,
}

impl<'a> Mangler<'a> {
  pub fn new(allocator: &'a Allocator) -> Self {
    Self {
      allocator,
      non_mangable: FxHashSet::default(),
      equality_groups: Vec::new(),
      uniqueness_groups: Vec::new(),
      atoms: FxHashMap::default(),
    }
  }

  pub fn mark_equality(&mut self, eq: bool, a: MangleAtom, b: MangleAtom) {
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
      let Mangler { atoms: constraints, equality_groups, uniqueness_groups, .. } = self;
      let existing = get_two_mut_from_map_or_insert(constraints, a, b);
      if eq {
        match existing {
          ((Some(ia), _), (Some(ib), _)) => {
            if ia != ib {
              let ((ga, _), (gb, _)) = get_two_mut_from_vec_unwrap(equality_groups, *ia, *ib);
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
            equality_groups[*ia].0.push(b);
          }
          ((ia @ None, _), (Some(ib), _)) => {
            *ia = Some(*ib);
            equality_groups[*ib].0.push(a);
          }
          ((ia @ None, _), (ib @ None, _)) => {
            let index = equality_groups.len();
            *ia = Some(index);
            *ib = Some(index);
            equality_groups.push((vec![a, b], None));
          }
        }
      } else {
        let index = uniqueness_groups.len();
        let ((_, uniq_groups_a), (_, uniq_groups_b)) = existing;
        uniq_groups_a.push(index);
        uniq_groups_b.push(index);
        uniqueness_groups.push((vec![a, b], 0));
      }
    }
  }

  /// Returns `Some` if they are mangable.
  pub fn mark_atoms_related(
    &mut self,
    a: MangleAtom,
    b: MangleAtom,
  ) -> Option<(&mut MangleAtomGroups, &mut MangleAtomGroups)> {
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
      None
    } else {
      Some(get_two_mut_from_map_or_insert(&mut self.atoms, a, b))
    }
  }

  fn mark_atom_non_mangable(&mut self, atom: MangleAtom) {
    if self.non_mangable.insert(atom) {
      if let Some((eq_group, uniq_groups)) = self.atoms.remove(&atom) {
        if let Some(index) = eq_group {
          for atom in mem::take(&mut self.equality_groups[index].0) {
            self.mark_atom_non_mangable(atom);
          }
        }
        for index in uniq_groups {
          for atom in mem::take(&mut self.uniqueness_groups[index].0) {
            self.mark_atom_non_mangable(atom);
          }
        }
      }
    }
  }

  pub fn resolve(&mut self, atom: MangleAtom) -> Option<&'a str> {
    if self.non_mangable.contains(&atom) {
      None
    } else {
      Some(if let Some((eq_group, uniq_groups)) = self.atoms.get(&atom) {
        if let Some(eq_group) = eq_group {
          self.resolve_eq_group(*eq_group)
        } else {
          let n = uniq_groups.iter().map(|&index| self.uniqueness_groups[index].1).max().unwrap();
          for &index in uniq_groups {
            self.uniqueness_groups[index].1 = n + 1;
          }
          self.allocator.alloc(get_mangled_name(n))
        }
      } else {
        // No constraints
        // This is quite weird, isn't it?
        "a"
      })
    }
  }

  fn resolve_eq_group(&mut self, index: usize) -> &'a str {
    let Mangler { equality_groups, uniqueness_groups, atoms: constraints, .. } = self;
    let (atoms, resolved_name) = &mut equality_groups[index];
    resolved_name.get_or_insert_with(|| {
      let mut n = 0;
      let mut related_uniq_groups = vec![];
      for atom in atoms {
        let (_, uniq_groups) = constraints.get(atom).unwrap();
        for index in uniq_groups {
          related_uniq_groups.push(*index);
          n = n.max(uniqueness_groups[*index].1);
        }
      }
      for index in related_uniq_groups {
        uniqueness_groups[index].1 = n + 1;
      }
      self.allocator.alloc(get_mangled_name(n))
    })
  }
}
