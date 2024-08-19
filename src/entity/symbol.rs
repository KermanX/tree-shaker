use std::{
  cell::LazyCell,
  rc::Rc,
  sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug, Clone)]
pub struct SymbolEntity {
  /// `0` is reserved for UnknownSymbol
  pub id: usize,
  pub name: Option<Rc<String>>,
  /// Prevent construction outside of this module
  _private: (),
}

impl PartialEq for SymbolEntity {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl SymbolEntity {
  pub fn new(name: Option<Rc<String>>) -> Self {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    Self { id, name, _private: () }
  }
}

pub const SYMBOL_ITERATOR: LazyCell<SymbolEntity> =
  LazyCell::new(|| SymbolEntity::new(Some(Rc::new("Symbol.iterator".to_string()))));
