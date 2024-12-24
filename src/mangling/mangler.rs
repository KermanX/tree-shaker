use std::mem;

use super::{utils::get_mangled_name, MangleAtom};
use oxc::allocator::Allocator;
use oxc_index::IndexVec;
use rustc_hash::{FxHashMap, FxHashSet};

oxc_index::define_index_type! {
  pub struct IdentityGroupId = u32;
  DISABLE_MAX_INDEX_CHECK = cfg!(not(debug_assertions));
}

oxc_index::define_index_type! {
  pub struct UniquenessGroupId = u32;
  DISABLE_MAX_INDEX_CHECK = cfg!(not(debug_assertions));
}

pub type MangleAtomGroups = (Option<IdentityGroupId>, FxHashSet<UniquenessGroupId>);

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

  pub fn mark_atom_non_mangable(&mut self, atom: MangleAtom) {
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

  pub fn mark_uniqueness_group_non_mangable(&mut self, group: UniquenessGroupId) {
    for atom in mem::take(&mut self.uniqueness_groups[group].0) {
      self.mark_atom_non_mangable(atom);
    }
  }

  pub fn add_to_uniqueness_group(&mut self, group: UniquenessGroupId, atom: MangleAtom) {
    self.atoms.entry(atom).or_default().1.insert(group);
    self.uniqueness_groups[group].0.push(atom);
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
