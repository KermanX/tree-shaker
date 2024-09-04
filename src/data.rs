use crate::entity::dep::EntityDepNode;
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::Debug;

pub struct DataPlaceholder<'a> {
  _phantom: std::marker::PhantomData<&'a ()>,
}

pub type ExtraData<'a> = FxHashMap<usize, Box<DataPlaceholder<'a>>>;

pub type ReferredNodes<'a> = FxHashSet<EntityDepNode<'a>>;

pub fn get_ptr_of_node(node: &impl Debug) -> usize {
  node as *const _ as usize
}
