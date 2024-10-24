use crate::{dep::DepId, entity::FunctionEntitySource};
use oxc::semantic::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeSet;

pub struct DataPlaceholder<'a> {
  _phantom: std::marker::PhantomData<&'a ()>,
}

pub type ExtraData<'a> = FxHashMap<DepId, Box<DataPlaceholder<'a>>>;

pub type ReferredNodes<'a> = FxHashMap<DepId, usize>;

pub type VarDeclarations<'a> = FxHashMap<FunctionEntitySource<'a>, FxHashSet<SymbolId>>;

pub type Diagnostics = BTreeSet<String>;

#[derive(Debug, Default)]
pub struct StatementVecData {
  pub last_stmt: Option<usize>,
}
