use std::collections::HashMap;

pub fn get_two_mut_from_map_or_insert<K, V, S>(
  map: &mut HashMap<K, V, S>,
  key1: K,
  key2: K,
) -> (&mut V, &mut V)
where
  K: std::cmp::Eq + std::hash::Hash,
  V: Default,
  S: std::hash::BuildHasher,
{
  unsafe {
    let map = map as *mut HashMap<K, V, S>;
    ((*map).entry(key1).or_default(), (*map).entry(key2).or_default())
  }
}

pub fn get_two_mut_from_vec_unwrap<V>(
  vec: &mut Vec<V>,
  index1: usize,
  index2: usize,
) -> (&mut V, &mut V) {
  unsafe {
    let vec = vec as *mut Vec<V>;
    (&mut (*vec)[index1], &mut (*vec)[index2])
  }
}
