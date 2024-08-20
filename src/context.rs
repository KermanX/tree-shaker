use crate::{builtins::global_this::GLOBAL_THIS, entity::Entity};
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;
use std::rc::Rc;

pub struct Context {
  pub(crate) this: Rc<Entity>,
  pub(crate) vars: FxHashMap<SymbolId, Rc<Entity>>,
}

impl Context {
  pub fn new() -> Self {
    Context {
      this: GLOBAL_THIS.clone(),
      vars: FxHashMap::default(),
    }
  }
}
