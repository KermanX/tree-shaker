use oxc_index::{Idx, IndexVec};
use std::collections::HashMap;

pub fn get_two_mut_from_map_or_insert<K, V, S>(
  map: &mut HashMap<K, V, S>,
  key1: K,
  key2: K,
) -> (&mut V, &mut V)
where
  K: std::cmp::Eq + std::hash::Hash + std::fmt::Debug + Copy,
  V: Default,
  S: std::hash::BuildHasher,
{
  debug_assert_ne!(key1, key2);
  unsafe {
    map.entry(key1).or_default();
    map.entry(key2).or_default();
    let map_ptr = map as *mut HashMap<K, V, S>;
    ((*map_ptr).get_mut(&key1).unwrap(), (*map_ptr).get_mut(&key2).unwrap())
  }
}

pub fn get_two_mut_from_vec_unwrap<K, V>(
  vec: &mut IndexVec<K, V>,
  index1: K,
  index2: K,
) -> (&mut V, &mut V)
where
  K: Idx,
{
  unsafe {
    let vec = vec as *mut IndexVec<K, V>;
    (&mut (*vec)[index1], &mut (*vec)[index2])
  }
}
