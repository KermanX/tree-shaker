use super::MangleAtom;
use crate::utils::{get_mangled_name, get_two_mut_from_map_or_insert, get_two_mut_from_vec_unwrap};
use oxc::allocator::Allocator;
use oxc_index::IndexVec;
use rustc_hash::{FxHashMap, FxHashSet};
use std::mem;

oxc_index::define_index_type! {
  pub struct IdentityGroupId = u32;
  DISABLE_MAX_INDEX_CHECK = cfg!(not(debug_assertions));
}

oxc_index::define_index_type! {
  pub struct UniquenessGroupId = u32;
  DISABLE_MAX_INDEX_CHECK = cfg!(not(debug_assertions));
}

pub type MangleAtomGroups = (Option<IdentityGroupId>, Vec<UniquenessGroupId>);

pub struct Mangler<'a> {
  pub allocator: &'a Allocator,

  pub non_mangable: FxHashSet<MangleAtom>,

  /// (atoms, resolved_name)[]
  pub identity_groups: IndexVec<IdentityGroupId, (Vec<MangleAtom>, Option<&'a str>)>,
  /// (atoms, used_names)[]
  pub uniqueness_groups: IndexVec<UniquenessGroupId, (Vec<MangleAtom>, usize)>,

  /// atom -> (identity_group_index, uniqueness_group_index)
  pub atoms: FxHashMap<MangleAtom, MangleAtomGroups>,
}

impl<'a> Mangler<'a> {
  pub fn new(allocator: &'a Allocator) -> Self {
    Self {
      allocator,
      non_mangable: FxHashSet::default(),
      identity_groups: IndexVec::new(),
      uniqueness_groups: IndexVec::new(),
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
        uniq_groups_a.push(id);
        uniq_groups_b.push(id);
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
      if let Some((identity_group, uniqueness_groups)) = self.atoms.remove(&atom) {
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
  }

  pub fn resolve(&mut self, atom: MangleAtom) -> Option<&'a str> {
    if self.non_mangable.contains(&atom) {
      None
    } else {
      Some(if let Some((identity_group, uniqueness_groups)) = self.atoms.get(&atom) {
        if let Some(eq_group) = identity_group {
          self.resolve_identity_group(*eq_group)
        } else {
          let mut n =
            uniqueness_groups.iter().map(|&index| self.uniqueness_groups[index].1).max().unwrap();
          let name = get_mangled_name(&mut n);
          for &index in uniqueness_groups {
            self.uniqueness_groups[index].1 = n;
          }
          self.allocator.alloc(name)
        }
      } else {
        // No constraints
        // This is quite weird, isn't it?
        "a"
      })
    }
  }

  fn resolve_identity_group(&mut self, id: IdentityGroupId) -> &'a str {
    let Mangler { identity_groups, uniqueness_groups, atoms: constraints, .. } = self;
    let (atoms, resolved_name) = &mut identity_groups[id];
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
      let name = get_mangled_name(&mut n);
      for index in related_uniq_groups {
        uniqueness_groups[index].1 = n;
      }
      self.allocator.alloc(name)
    })
  }
}
