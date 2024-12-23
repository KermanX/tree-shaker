use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableTrait},
  utils::{get_mangled_name, get_two_mut_from_map_or_insert, get_two_mut_from_vec_unwrap},
};
use oxc::allocator::Allocator;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
  mem,
  num::{NonZero, NonZeroU64},
  sync::atomic::{AtomicU64, Ordering},
};

static MANGLE_ATOM_COUNT: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MangleAtom(NonZeroU64);

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MangleConstraint {
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
      MangleConstraint::Eq(a, b) => MangleConstraint::Neq(a, b),
      MangleConstraint::Neq(a, b) => MangleConstraint::Eq(a, b),
      MangleConstraint::Multiple(c) => {
        MangleConstraint::Multiple(c.into_iter().map(Self::negate).collect())
      }
    }
  }

  fn into_pair(self) -> (MangleAtom, MangleAtom) {
    match self {
      MangleConstraint::Eq(a, b) => (a, b),
      MangleConstraint::Neq(a, b) => (a, b),
      MangleConstraint::Multiple(_) => {
        unreachable!("Multiple constraints cannot be converted to pair")
      }
    }
  }
}

impl<'a> ConsumableTrait<'a> for &'a MangleConstraint {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    match *self {
      MangleConstraint::Multiple(cs) => {
        for constraint in cs {
          constraint.consume(analyzer);
        }
      }
      c => {
        analyzer.mangler.constraints.insert(c.clone());
      }
    }
  }

  fn cloned(&self) -> Consumable<'a> {
    unreachable!("MangleConstraint cannot be cloned")
  }
}

#[derive(Debug, Default)]
pub struct AnalyzerMangler {
  pub non_mangable: FxHashSet<MangleAtom>,
  constraints: FxHashSet<MangleConstraint>,
}

pub struct TransformerMangler<'a> {
  allocator: &'a Allocator,

  non_mangable: FxHashSet<MangleAtom>,

  /// (atoms, resolved_name)[]
  equality_groups: Vec<(Vec<MangleAtom>, Option<&'a str>)>,
  /// (atoms, used_names)[]
  uniqueness_groups: Vec<(Vec<MangleAtom>, usize)>,

  /// atom -> (equality_group_index, uniqueness_group_index)
  constraints: FxHashMap<MangleAtom, (Option<usize>, Vec<usize>)>,
}

impl AnalyzerMangler {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn into_transformer(self, allocator: &Allocator) -> TransformerMangler<'_> {
    let AnalyzerMangler { non_mangable, constraints } = self;

    let mut transformer = TransformerMangler {
      allocator,
      non_mangable,
      equality_groups: Vec::new(),
      uniqueness_groups: Vec::new(),
      constraints: FxHashMap::default(),
    };

    for constraint in constraints {
      let (a, b) = constraint.clone().into_pair();
      debug_assert_ne!(a, b);
      let a_is_non_mangable = transformer.non_mangable.contains(&a);
      let b_is_non_mangable = transformer.non_mangable.contains(&b);
      if a_is_non_mangable || b_is_non_mangable {
        if !a_is_non_mangable {
          transformer.mark_atom_non_mangable(a);
        }
        if !b_is_non_mangable {
          transformer.mark_atom_non_mangable(b);
        }
      } else {
        let TransformerMangler { constraints, equality_groups, uniqueness_groups, .. } =
          &mut transformer;
        let existing = get_two_mut_from_map_or_insert(constraints, a, b);
        match constraint {
          MangleConstraint::Eq(a, b) => match existing {
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
          },
          MangleConstraint::Neq(a, b) => {
            let index = uniqueness_groups.len();
            let ((_, uniq_groups_a), (_, uniq_groups_b)) = existing;
            uniq_groups_a.push(index);
            uniq_groups_b.push(index);
            uniqueness_groups.push((vec![a, b], 0));
          }
          MangleConstraint::Multiple(_) => unreachable!(),
        }
      }
    }

    // println!("Non-mangable: {:?}", transformer.non_mangable);
    // println!("Equality groups: {:?}", transformer.equality_groups);
    // println!("Uniqueness groups: {:?}", transformer.uniqueness_groups);
    // println!("Constraints: {:?}", transformer.constraints);

    transformer
  }
}

impl<'a> TransformerMangler<'a> {
  fn mark_atom_non_mangable(&mut self, atom: MangleAtom) {
    if self.non_mangable.insert(atom) {
      if let Some((eq_group, uniq_groups)) = self.constraints.remove(&atom) {
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
      Some(if let Some((eq_group, uniq_groups)) = self.constraints.get(&atom) {
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
    let TransformerMangler { equality_groups, uniqueness_groups, constraints, .. } = self;
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
