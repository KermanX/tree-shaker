use crate::{ast::AstType2, entity::dep::EntityDepNode};
use rustc_hash::{FxHashMap, FxHashSet};

pub struct DataPlaceholder<'a> {
  _phantom: std::marker::PhantomData<&'a ()>,
}

pub type ExtraData<'a> = FxHashMap<(AstType2, usize), Box<DataPlaceholder<'a>>>;

pub type ReferredNodes<'a> = FxHashSet<EntityDepNode<'a>>;

#[derive(Debug, Default)]
pub struct StatementVecData {
  pub last_stmt: Option<usize>,
}

pub fn get_node_ptr<T>(node: &T) -> usize {
  node as *const T as usize
}
