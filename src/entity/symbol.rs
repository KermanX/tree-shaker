use dashmap::DashMap;
use std::{
  cell::LazyCell,
  sync::{
    atomic::{AtomicUsize, Ordering},
    LazyLock,
  },
};

#[derive(Debug, Clone)]
pub struct SymbolEntity {
  /// `0` is reserved for unknown
  pub id: usize,
  /// Prevent construction outside of this module
  _private: (),
}

impl PartialEq for SymbolEntity {
  fn eq(&self, other: &Self) -> bool {
    self.id != 0 && self.id == other.id
  }
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
static FOR_MAP: LazyLock<DashMap<String, usize>> = LazyLock::new(DashMap::default);

impl SymbolEntity {
  pub fn new() -> Self {
    let id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    Self { id, _private: () }
  }

  pub fn new_for(name: String) -> Self {
    let id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    FOR_MAP.insert(name, id);
    Self { id, _private: () }
  }
}

pub const SYMBOL_ITERATOR: LazyCell<SymbolEntity> = LazyCell::new(SymbolEntity::new);
